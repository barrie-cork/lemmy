"""
GitHub Models API rate limit tracker for free tier compliance.
Prevents exceeding 150 req/day (low tier) and 50 req/day (high tier).
"""

import json
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path


class RateLimitTracker:
    """Track GitHub Models API usage to stay within free tier limits"""

    LIMITS = {
        "low": {"per_minute": 15, "per_day": 150},
        "high": {"per_minute": 10, "per_day": 50},
    }

    # Cap the in-memory request log to keep _save_usage() bounded.
    MAX_REQUEST_HISTORY = 200

    def __init__(self, cache_file=".github/cache/api_usage.json"):
        self.cache_file = Path(cache_file)
        self.cache_file.parent.mkdir(parents=True, exist_ok=True)
        self.usage = self._load_usage()

    def _load_usage(self):
        """Load usage data from cache file"""
        if self.cache_file.exists():
            try:
                with open(self.cache_file) as f:
                    data = json.load(f)
                    # Reset if new day (UTC so self-hosted runners don't drift)
                    if data.get("date") != str(datetime.now(timezone.utc).date()):
                        return self._new_usage_record()
                    return data
            except (json.JSONDecodeError, KeyError):
                pass
        return self._new_usage_record()

    def _new_usage_record(self):
        """Create new usage record for current day (UTC)"""
        return {
            "date": str(datetime.now(timezone.utc).date()),
            "low": 0,
            "high": 0,
            "requests": [],
        }

    def can_make_request(self, tier="low"):
        """
        Check if a request can be made within rate limits.

        Enforces both ``per_day`` and ``per_minute`` caps. Unknown tiers raise
        ``ValueError`` — the CLI converts that into a distinct exit code so
        workflows can tell a bad arg from an exhausted quota.
        """
        if tier not in self.LIMITS:
            raise ValueError(f"Unknown tier: {tier!r}. Expected one of {sorted(self.LIMITS)}")

        # Daily cap
        current_day = self.usage.get(tier, 0)
        if current_day >= self.LIMITS[tier]["per_day"]:
            return False

        # Per-minute cap: count entries for this tier in the last 60 seconds.
        # Timestamps are ISO-8601 UTC strings written by record_request.
        minute_limit = self.LIMITS[tier]["per_minute"]
        window_start = datetime.now(timezone.utc) - timedelta(seconds=60)
        recent = 0
        for entry in self.usage.get("requests", []):
            if entry.get("tier") != tier:
                continue
            ts_raw = entry.get("timestamp")
            if not ts_raw:
                continue
            try:
                ts = datetime.fromisoformat(ts_raw)
            except ValueError:
                continue
            # Legacy naive timestamps: assume UTC.
            if ts.tzinfo is None:
                ts = ts.replace(tzinfo=timezone.utc)
            if ts >= window_start:
                recent += 1
        if recent >= minute_limit:
            return False

        return True

    def record_request(self, tier="low", model="unknown", tokens_used=0):
        """Record API request. Unknown tiers raise ``ValueError``."""
        if tier not in self.LIMITS:
            raise ValueError(f"Unknown tier: {tier!r}. Expected one of {sorted(self.LIMITS)}")

        self.usage[tier] = self.usage.get(tier, 0) + 1
        self.usage["requests"].append(
            {
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "tier": tier,
                "model": model,
                "tokens": tokens_used,
            }
        )
        # Keep the history bounded so _save_usage() stays O(1) in file size.
        if len(self.usage["requests"]) > self.MAX_REQUEST_HISTORY:
            self.usage["requests"] = self.usage["requests"][-self.MAX_REQUEST_HISTORY :]
        self._save_usage()

    def _save_usage(self):
        """Save usage data to cache file"""
        with open(self.cache_file, "w") as f:
            json.dump(self.usage, f, indent=2)

    def get_remaining(self, tier="low"):
        """Get remaining requests for the day"""
        used = self.usage.get(tier, 0)
        limit = self.LIMITS[tier]["per_day"]
        return limit - used

    def get_stats(self):
        """Get usage statistics (GitHub Models only)."""
        return {
            "date": self.usage["date"],
            "low_tier": {
                "used": self.usage.get("low", 0),
                "remaining": self.get_remaining("low"),
                "limit": self.LIMITS["low"]["per_day"],
            },
            "high_tier": {
                "used": self.usage.get("high", 0),
                "remaining": self.get_remaining("high"),
                "limit": self.LIMITS["high"]["per_day"],
            },
            "total_requests": len(self.usage.get("requests", [])),
        }


def main():
    """CLI interface for rate limit tracker"""
    tracker = RateLimitTracker()

    if len(sys.argv) < 2:
        print(
            "Usage: python rate_limit_tracker.py [check|record|stats] [tier] [model] [tokens]",
            file=sys.stderr,
        )
        sys.exit(2)

    command = sys.argv[1]

    if command == "check":
        tier = sys.argv[2] if len(sys.argv) > 2 else "low"
        try:
            can_request = tracker.can_make_request(tier)
        except ValueError as e:
            print(f"error: {e}", file=sys.stderr)
            # Exit 2 = bad arg, distinct from exit 1 = quota exhausted.
            sys.exit(2)
        remaining = tracker.get_remaining(tier)
        print(f"can_request={str(can_request).lower()}")
        print(f"remaining={remaining}")
        sys.exit(0 if can_request else 1)

    elif command == "record":
        tier = sys.argv[2] if len(sys.argv) > 2 else "low"
        model = sys.argv[3] if len(sys.argv) > 3 else "unknown"
        tokens = int(sys.argv[4]) if len(sys.argv) > 4 else 0
        try:
            tracker.record_request(tier, model, tokens)
        except ValueError as e:
            print(f"error: {e}", file=sys.stderr)
            sys.exit(2)
        sys.exit(0)

    elif command == "stats":
        try:
            stats = tracker.get_stats()
        except Exception as e:
            # Return valid JSON even on error so the workflow can parse it.
            stats = {
                "date": str(datetime.now(timezone.utc).date()),
                "low_tier": {"used": 0, "remaining": 150, "limit": 150},
                "high_tier": {"used": 0, "remaining": 50, "limit": 50},
                "total_requests": 0,
                "error": str(e),
            }
        print(json.dumps(stats, indent=2))
        sys.exit(0)

    else:
        print(f"Unknown command: {command}", file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
