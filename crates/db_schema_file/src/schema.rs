// @generated automatically by Diesel CLI.

pub mod sql_types {
  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "actor_type_enum"))]
  pub struct ActorTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "appeal_requester_role"))]
  pub struct AppealRequesterRole;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "appeal_status"))]
  pub struct AppealStatus;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "attestation_type"))]
  pub struct AttestationType;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "case_severity"))]
  pub struct CaseSeverity;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "case_status"))]
  pub struct CaseStatus;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "case_status_tier"))]
  pub struct CaseStatusTier;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "case_target_type"))]
  pub struct CaseTargetType;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "comment_sort_type_enum"))]
  pub struct CommentSortTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "community_follower_state"))]
  pub struct CommunityFollowerState;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "community_notifications_mode_enum"))]
  pub struct CommunityNotificationsModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "community_visibility"))]
  pub struct CommunityVisibility;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "evidence_visibility"))]
  pub struct EvidenceVisibility;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "federation_inbox_admin_action_enum"))]
  pub struct FederationInboxAdminActionEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "federation_mode_enum"))]
  pub struct FederationModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "federation_peer_trust_enum"))]
  pub struct FederationPeerTrustEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "image_mode_enum"))]
  pub struct ImageModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "jury_assignment_role"))]
  pub struct JuryAssignmentRole;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "jury_assignment_status"))]
  pub struct JuryAssignmentStatus;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "jury_constraint_relaxation_reason"))]
  pub struct JuryConstraintRelaxationReason;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "jury_decision"))]
  pub struct JuryDecision;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "listing_type_enum"))]
  pub struct ListingTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "ltree"))]
  pub struct Ltree;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "membership_state"))]
  pub struct MembershipState;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "modlog_kind"))]
  pub struct ModlogKind;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "notification_type_enum"))]
  pub struct NotificationTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "post_listing_mode_enum"))]
  pub struct PostListingModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "post_notifications_mode_enum"))]
  pub struct PostNotificationsModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "post_sort_type_enum"))]
  pub struct PostSortTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "registration_mode_enum"))]
  pub struct RegistrationModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "reputation_dimension"))]
  pub struct ReputationDimension;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "reputation_event_source_type"))]
  pub struct ReputationEventSourceType;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "sanction_action"))]
  pub struct SanctionAction;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "sanction_scope"))]
  pub struct SanctionScope;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "severity_tier"))]
  pub struct SeverityTier;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "tag_color_enum"))]
  pub struct TagColorEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "vote_show_enum"))]
  pub struct VoteShowEnum;
}

diesel::table! {
    actor_pseudonym (id) {
        id -> Int4,
        person_id -> Int4,
        pseudonym -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AppealStatus;
    use super::sql_types::AppealRequesterRole;

    appeal (id) {
        id -> Int4,
        case_id -> Int4,
        requester_id -> Int4,
        reason -> Text,
        status -> AppealStatus,
        created_at -> Timestamptz,
        decided_at -> Nullable<Timestamptz>,
        requester_role -> AppealRequesterRole,
        panel_size_snapshot -> Nullable<Int4>,
        threshold_count_snapshot -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::EvidenceVisibility;

    case_evidence (id) {
        id -> Int4,
        case_id -> Int4,
        uploader_id -> Int4,
        storage_key -> Text,
        sha256 -> Text,
        mime_type -> Text,
        visibility -> EvidenceVisibility,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_ltree::sql_types::Ltree;

    comment (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        content -> Text,
        removed -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        path -> Ltree,
        distinguished -> Bool,
        language_id -> Int4,
        score -> Int4,
        upvotes -> Int4,
        downvotes -> Int4,
        child_count -> Int4,
        hot_rank -> Float4,
        controversy_rank -> Float4,
        report_count -> Int2,
        unresolved_report_count -> Int2,
        federation_pending -> Bool,
        locked -> Bool,
        community_id -> Int4,
    }
}

diesel::table! {
    comment_actions (person_id, comment_id) {
        voted_at -> Nullable<Timestamptz>,
        saved_at -> Nullable<Timestamptz>,
        person_id -> Int4,
        comment_id -> Int4,
        vote_is_upvote -> Nullable<Bool>,
    }
}

diesel::table! {
    comment_report (id) {
        id -> Int4,
        creator_id -> Int4,
        comment_id -> Int4,
        original_comment_text -> Text,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        violates_instance_rules -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityVisibility;

    community (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 50]
        title -> Varchar,
        sidebar -> Nullable<Text>,
        removed -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
        nsfw -> Bool,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamptz,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        #[max_length = 255]
        followers_url -> Nullable<Varchar>,
        #[max_length = 255]
        inbox_url -> Varchar,
        posting_restricted_to_mods -> Bool,
        instance_id -> Int4,
        #[max_length = 255]
        moderators_url -> Nullable<Varchar>,
        #[max_length = 255]
        featured_url -> Nullable<Varchar>,
        visibility -> CommunityVisibility,
        #[max_length = 150]
        summary -> Nullable<Varchar>,
        random_number -> Int2,
        subscribers -> Int4,
        posts -> Int4,
        comments -> Int4,
        users_active_day -> Int4,
        users_active_week -> Int4,
        users_active_month -> Int4,
        users_active_half_year -> Int4,
        hot_rank -> Float4,
        subscribers_local -> Int4,
        interactions_month -> Int4,
        report_count -> Int2,
        unresolved_report_count -> Int2,
        local_removed -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityFollowerState;
    use super::sql_types::CommunityNotificationsModeEnum;

    community_actions (person_id, community_id) {
        followed_at -> Nullable<Timestamptz>,
        blocked_at -> Nullable<Timestamptz>,
        became_moderator_at -> Nullable<Timestamptz>,
        received_ban_at -> Nullable<Timestamptz>,
        ban_expires_at -> Nullable<Timestamptz>,
        person_id -> Int4,
        community_id -> Int4,
        follow_state -> Nullable<CommunityFollowerState>,
        follow_approver_id -> Nullable<Int4>,
        notifications -> Nullable<CommunityNotificationsModeEnum>,
    }
}

diesel::table! {
    community_community_follow (community_id, target_id) {
        target_id -> Int4,
        community_id -> Int4,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    community_language (community_id, language_id) {
        community_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    community_report (id) {
        id -> Int4,
        creator_id -> Int4,
        community_id -> Int4,
        original_community_name -> Text,
        original_community_title -> Text,
        original_community_summary -> Nullable<Text>,
        original_community_sidebar -> Nullable<Text>,
        original_community_icon -> Nullable<Text>,
        original_community_banner -> Nullable<Text>,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TagColorEnum;

    community_tag (id) {
        id -> Int4,
        ap_id -> Text,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        display_name -> Nullable<Varchar>,
        #[max_length = 150]
        summary -> Nullable<Varchar>,
        community_id -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
        color -> TagColorEnum,
    }
}

diesel::table! {
    custom_emoji (id) {
        id -> Int4,
        #[max_length = 128]
        shortcode -> Varchar,
        image_url -> Text,
        alt_text -> Text,
        category -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    custom_emoji_keyword (custom_emoji_id, keyword) {
        custom_emoji_id -> Int4,
        #[max_length = 128]
        keyword -> Varchar,
    }
}

diesel::table! {
    email_verification (id) {
        id -> Int4,
        local_user_id -> Int4,
        email -> Text,
        verification_token -> Text,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    endorsement (id) {
        id -> Int4,
        from_person_id -> Int4,
        to_person_id -> Int4,
        community_id -> Nullable<Int4>,
        created_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    federation_allowlist (instance_id) {
        instance_id -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AttestationType;
    use super::sql_types::FederationInboxAdminActionEnum;
    use super::sql_types::FederationPeerTrustEnum;

    federation_attestation (id) {
        id -> Int4,
        actor_url -> Text,
        subject_url -> Text,
        attestation_type -> AttestationType,
        valid_until -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        signature -> Text,
        // v1-federation-inbound-a additions:
        source_instance -> Nullable<Text>,
        received_at -> Nullable<Timestamptz>,
        peer_trust_level_at_receipt -> Nullable<FederationPeerTrustEnum>,
        admin_reviewed_at -> Nullable<Timestamptz>,
        admin_action -> FederationInboxAdminActionEnum,
        dismissal_rationale -> Nullable<Text>,
    }
}

diesel::table! {
    federation_blocklist (instance_id) {
        instance_id -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
    }
}

// v1-federation-inbound-a additions:
diesel::table! {
    federation_inbox_dropped_log (id) {
        id -> Int8,
        source_instance -> Text,
        activity_id -> Nullable<Text>,
        drop_reason -> Text,
        payload_excerpt -> Nullable<Text>,
        dropped_at -> Timestamptz,
    }
}

// v1-federation-inbound-a additions:
diesel::table! {
    federation_inbox_nonce (peer_instance, activity_id) {
        peer_instance -> Text,
        activity_id -> Text,
        seen_at -> Timestamptz,
    }
}

// v1-federation-inbound-a additions:
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FederationPeerTrustEnum;

    federation_peer (instance_id) {
        instance_id -> Int4,
        trust_level -> FederationPeerTrustEnum,
        added_at -> Timestamptz,
        added_by_actor -> Nullable<Text>,
        notes -> Jsonb,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    federation_queue_state (instance_id) {
        instance_id -> Int4,
        last_successful_id -> Nullable<Int8>,
        fail_count -> Int4,
        last_retry_at -> Nullable<Timestamptz>,
        last_successful_published_time_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    governance_config (id) {
        id -> Int4,
        scope -> Text,
        key -> Text,
        value_type -> Text,
        value_int -> Nullable<Int8>,
        value_float -> Nullable<Float8>,
        value_bool -> Nullable<Bool>,
        value_text -> Nullable<Text>,
        valid_from -> Timestamptz,
        updated_by -> Nullable<Int4>,
    }
}

diesel::table! {
    governance_messaging_config (id) {
        id -> Int4,
        scope -> Text,
        key -> Text,
        value_type -> Text,
        value_int -> Nullable<Int8>,
        value_bool -> Nullable<Bool>,
        value_text -> Nullable<Text>,
        valid_from -> Timestamptz,
        updated_by -> Nullable<Int4>,
    }
}

diesel::table! {
    governance_log (id) {
        id -> Int8,
        prev_hash -> Bytea,
        entry_hash -> Bytea,
        entry_kind -> Text,
        payload -> Jsonb,
        actor_pseudonym -> Nullable<Text>,
        created_at -> Timestamptz,
        signature -> Nullable<Bytea>,
    }
}

diesel::table! {
    image_details (link) {
        link -> Text,
        width -> Int4,
        height -> Int4,
        content_type -> Text,
        #[max_length = 50]
        blurhash -> Nullable<Varchar>,
    }
}

diesel::table! {
    instance (id) {
        id -> Int4,
        #[max_length = 255]
        domain -> Varchar,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        software -> Nullable<Varchar>,
        #[max_length = 255]
        version -> Nullable<Varchar>,
    }
}

diesel::table! {
    instance_actions (person_id, instance_id) {
        blocked_communities_at -> Nullable<Timestamptz>,
        person_id -> Int4,
        instance_id -> Int4,
        received_ban_at -> Nullable<Timestamptz>,
        ban_expires_at -> Nullable<Timestamptz>,
        blocked_persons_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JuryAssignmentStatus;
    use super::sql_types::JuryAssignmentRole;

    jury_assignment (id) {
        id -> Int4,
        case_id -> Int4,
        person_id -> Int4,
        status -> JuryAssignmentStatus,
        selected_at -> Timestamptz,
        responded_at -> Nullable<Timestamptz>,
        submitted_at -> Nullable<Timestamptz>,
        selected_under_constraints -> Nullable<Jsonb>,
        role -> JuryAssignmentRole,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JuryConstraintRelaxationReason;

    jury_constraint_violation_log (id) {
        id -> Int4,
        case_id -> Int4,
        constraint_name -> Text,
        reason_code -> JuryConstraintRelaxationReason,
        relaxation_metadata -> Nullable<Jsonb>,
        pool_size_at_relax -> Int4,
        panel_size_target -> Int4,
        relaxed_at -> Timestamptz,
    }
}

diesel::table! {
    jury_pool (id) {
        id -> Int4,
        community_id -> Nullable<Int4>,
        person_id -> Int4,
        eligible_from -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JuryDecision;

    jury_vote (id) {
        id -> Int4,
        case_id -> Int4,
        juror_id -> Int4,
        decision -> JuryDecision,
        rationale -> Nullable<Text>,
        submitted_at -> Timestamptz,
    }
}

diesel::table! {
    language (id) {
        id -> Int4,
        #[max_length = 3]
        code -> Varchar,
        name -> Text,
    }
}

diesel::table! {
    local_image (pictrs_alias) {
        pictrs_alias -> Text,
        published_at -> Timestamptz,
        person_id -> Nullable<Int4>,
        thumbnail_for_post_id -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ListingTypeEnum;
    use super::sql_types::RegistrationModeEnum;
    use super::sql_types::PostListingModeEnum;
    use super::sql_types::PostSortTypeEnum;
    use super::sql_types::CommentSortTypeEnum;
    use super::sql_types::FederationModeEnum;
    use super::sql_types::ImageModeEnum;

    local_site (id) {
        id -> Int4,
        site_id -> Int4,
        site_setup -> Bool,
        community_creation_admin_only -> Bool,
        email_verification_required -> Bool,
        application_question -> Nullable<Text>,
        private_instance -> Bool,
        default_theme -> Text,
        default_post_listing_type -> ListingTypeEnum,
        legal_information -> Nullable<Text>,
        application_email_admins -> Bool,
        slur_filter_regex -> Nullable<Text>,
        federation_enabled -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        registration_mode -> RegistrationModeEnum,
        reports_email_admins -> Bool,
        federation_signed_fetch -> Bool,
        default_post_listing_mode -> PostListingModeEnum,
        default_post_sort_type -> PostSortTypeEnum,
        default_comment_sort_type -> CommentSortTypeEnum,
        oauth_registration -> Bool,
        post_upvotes -> FederationModeEnum,
        post_downvotes -> FederationModeEnum,
        comment_upvotes -> FederationModeEnum,
        comment_downvotes -> FederationModeEnum,
        default_post_time_range_seconds -> Nullable<Int4>,
        nsfw_content_disallowed -> Bool,
        users -> Int4,
        posts -> Int4,
        comments -> Int4,
        communities -> Int4,
        users_active_day -> Int4,
        users_active_week -> Int4,
        users_active_month -> Int4,
        users_active_half_year -> Int4,
        email_notifications_disabled -> Bool,
        suggested_multi_community_id -> Nullable<Int4>,
        system_account -> Int4,
        default_items_per_page -> Int4,
        image_mode -> ImageModeEnum,
        image_proxy_bypass_domains -> Nullable<Text>,
        image_upload_timeout_seconds -> Int4,
        image_max_thumbnail_size -> Int4,
        image_max_avatar_size -> Int4,
        image_max_banner_size -> Int4,
        image_max_upload_size -> Int4,
        image_allow_video_uploads -> Bool,
        image_upload_disabled -> Bool,
    }
}

diesel::table! {
    local_site_rate_limit (local_site_id) {
        local_site_id -> Int4,
        message_max_requests -> Int4,
        message_interval_seconds -> Int4,
        post_max_requests -> Int4,
        post_interval_seconds -> Int4,
        register_max_requests -> Int4,
        register_interval_seconds -> Int4,
        image_max_requests -> Int4,
        image_interval_seconds -> Int4,
        comment_max_requests -> Int4,
        comment_interval_seconds -> Int4,
        search_max_requests -> Int4,
        search_interval_seconds -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        import_user_settings_max_requests -> Int4,
        import_user_settings_interval_seconds -> Int4,
    }
}

diesel::table! {
    local_site_url_blocklist (id) {
        id -> Int4,
        url -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PostSortTypeEnum;
    use super::sql_types::ListingTypeEnum;
    use super::sql_types::PostListingModeEnum;
    use super::sql_types::CommentSortTypeEnum;
    use super::sql_types::VoteShowEnum;

    local_user (id) {
        id -> Int4,
        person_id -> Int4,
        password_encrypted -> Nullable<Text>,
        email -> Nullable<Text>,
        show_nsfw -> Bool,
        theme -> Text,
        default_post_sort_type -> PostSortTypeEnum,
        default_listing_type -> ListingTypeEnum,
        #[max_length = 20]
        interface_language -> Varchar,
        show_avatars -> Bool,
        send_notifications_to_email -> Bool,
        show_bot_accounts -> Bool,
        show_read_posts -> Bool,
        email_verified -> Bool,
        accepted_application -> Bool,
        totp_2fa_secret -> Nullable<Text>,
        open_links_in_new_tab -> Bool,
        blur_nsfw -> Bool,
        infinite_scroll_enabled -> Bool,
        admin -> Bool,
        post_listing_mode -> PostListingModeEnum,
        totp_2fa_enabled -> Bool,
        animated_images_enabled -> Bool,
        collapse_bot_comments -> Bool,
        last_donation_notification_at -> Timestamptz,
        private_messages_enabled -> Bool,
        default_comment_sort_type -> CommentSortTypeEnum,
        auto_mark_fetched_posts_as_read -> Bool,
        hide_media -> Bool,
        default_post_time_range_seconds -> Nullable<Int4>,
        show_score -> Bool,
        show_upvotes -> Bool,
        show_downvotes -> VoteShowEnum,
        show_upvote_percentage -> Bool,
        show_person_votes -> Bool,
        default_items_per_page -> Int4,
    }
}

diesel::table! {
    local_user_keyword_block (local_user_id, keyword) {
        local_user_id -> Int4,
        #[max_length = 50]
        keyword -> Varchar,
    }
}

diesel::table! {
    local_user_language (local_user_id, language_id) {
        local_user_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    login_token (token) {
        token -> Text,
        user_id -> Int4,
        published_at -> Timestamptz,
        ip -> Nullable<Text>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CaseTargetType;
    use super::sql_types::CaseSeverity;
    use super::sql_types::CaseStatus;
    use super::sql_types::SeverityTier;
    use super::sql_types::CaseStatusTier;
    use super::sql_types::JuryDecision;

    moderation_case (id) {
        id -> Int4,
        community_id -> Nullable<Int4>,
        creator_id -> Nullable<Int4>,
        target_type -> CaseTargetType,
        target_post_id -> Nullable<Int4>,
        target_comment_id -> Nullable<Int4>,
        target_person_id -> Nullable<Int4>,
        target_community_id -> Nullable<Int4>,
        target_remote_url -> Nullable<Text>,
        reason_code -> Text,
        severity -> CaseSeverity,
        status -> CaseStatus,
        threshold_score -> Int8,
        opened_at -> Timestamptz,
        decided_at -> Nullable<Timestamptz>,
        closed_at -> Nullable<Timestamptz>,
        applied_config_snapshot -> Nullable<Jsonb>,
        rule_set_version_id -> Nullable<Int4>,
        severity_tier -> SeverityTier,
        status_tier -> CaseStatusTier,
        panel_size_snapshot -> Nullable<Int4>,
        quorum_snapshot -> Nullable<Int4>,
        threshold_count_snapshot -> Nullable<Int4>,
        appeal_window_expires_at -> Nullable<Timestamptz>,
        winning_decision -> Nullable<JuryDecision>,
        // v1-SL-a additions:
        grace_expires_at -> Nullable<Timestamptz>,
        liability_escape_reason -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ModlogKind;

    modlog (id) {
        id -> Int4,
        kind -> ModlogKind,
        is_revert -> Bool,
        mod_id -> Int4,
        reason -> Nullable<Text>,
        target_person_id -> Nullable<Int4>,
        target_community_id -> Nullable<Int4>,
        target_post_id -> Nullable<Int4>,
        target_comment_id -> Nullable<Int4>,
        target_instance_id -> Nullable<Int4>,
        expires_at -> Nullable<Timestamptz>,
        published_at -> Timestamptz,
        bulk_action_parent_id -> Nullable<Int4>,
    }
}

diesel::table! {
    multi_community (id) {
        id -> Int4,
        creator_id -> Int4,
        instance_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        title -> Nullable<Varchar>,
        #[max_length = 255]
        summary -> Nullable<Varchar>,
        local -> Bool,
        deleted -> Bool,
        ap_id -> Text,
        public_key -> Text,
        private_key -> Nullable<Text>,
        inbox_url -> Text,
        last_refreshed_at -> Timestamptz,
        following_url -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        subscribers -> Int4,
        subscribers_local -> Int4,
        communities -> Int4,
        sidebar -> Nullable<Text>,
    }
}

diesel::table! {
    multi_community_entry (multi_community_id, community_id) {
        multi_community_id -> Int4,
        community_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityFollowerState;

    multi_community_follow (person_id, multi_community_id) {
        multi_community_id -> Int4,
        person_id -> Int4,
        follow_state -> CommunityFollowerState,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NotificationTypeEnum;

    notification (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Nullable<Int4>,
        read -> Bool,
        published_at -> Timestamptz,
        kind -> NotificationTypeEnum,
        post_id -> Nullable<Int4>,
        private_message_id -> Nullable<Int4>,
        modlog_id -> Nullable<Int4>,
        creator_id -> Int4,
        instance_id -> Nullable<Int4>,
        community_id -> Nullable<Int4>,
    }
}

diesel::table! {
    oauth_account (oauth_provider_id, local_user_id) {
        local_user_id -> Int4,
        oauth_provider_id -> Int4,
        oauth_user_id -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    oauth_provider (id) {
        id -> Int4,
        display_name -> Text,
        issuer -> Text,
        authorization_endpoint -> Text,
        token_endpoint -> Text,
        userinfo_endpoint -> Text,
        id_claim -> Text,
        client_id -> Text,
        client_secret -> Text,
        scopes -> Text,
        auto_verify_email -> Bool,
        account_linking_enabled -> Bool,
        enabled -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        use_pkce -> Bool,
    }
}

diesel::table! {
    password_reset_request (id) {
        id -> Int4,
        token -> Text,
        published_at -> Timestamptz,
        local_user_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MembershipState;

    person (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 50]
        display_name -> Nullable<Varchar>,
        avatar -> Nullable<Text>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        ap_id -> Varchar,
        bio -> Nullable<Text>,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamptz,
        banner -> Nullable<Text>,
        deleted -> Bool,
        #[max_length = 255]
        inbox_url -> Varchar,
        matrix_user_id -> Nullable<Text>,
        bot_account -> Bool,
        instance_id -> Int4,
        post_count -> Int4,
        post_score -> Int4,
        comment_count -> Int4,
        comment_score -> Int4,
        membership_state -> MembershipState,
    }
}

diesel::table! {
    person_actions (person_id, target_id) {
        followed_at -> Nullable<Timestamptz>,
        blocked_at -> Nullable<Timestamptz>,
        person_id -> Int4,
        target_id -> Int4,
        follow_pending -> Nullable<Bool>,
        noted_at -> Nullable<Timestamptz>,
        note -> Nullable<Text>,
        voted_at -> Nullable<Timestamptz>,
        upvotes -> Nullable<Int4>,
        downvotes -> Nullable<Int4>,
    }
}

diesel::table! {
    person_content_combined (id) {
        published_at -> Timestamptz,
        creator_id -> Int4,
        post_id -> Int4,
        comment_id -> Nullable<Int4>,
        id -> Int4,
        community_id -> Int4,
    }
}

diesel::table! {
    person_liked_combined (id) {
        voted_at -> Timestamptz,
        id -> Int4,
        person_id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        comment_id -> Nullable<Int4>,
        vote_is_upvote -> Bool,
        community_id -> Int4,
    }
}

diesel::table! {
    person_saved_combined (id) {
        saved_at -> Timestamptz,
        person_id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        comment_id -> Nullable<Int4>,
        id -> Int4,
        community_id -> Int4,
    }
}

diesel::table! {
    post (id) {
        id -> Int4,
        #[max_length = 200]
        name -> Varchar,
        #[max_length = 2000]
        url -> Nullable<Varchar>,
        body -> Nullable<Text>,
        creator_id -> Int4,
        community_id -> Int4,
        removed -> Bool,
        locked -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
        nsfw -> Bool,
        embed_title -> Nullable<Text>,
        embed_description -> Nullable<Text>,
        thumbnail_url -> Nullable<Text>,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        embed_video_url -> Nullable<Text>,
        language_id -> Int4,
        featured_community -> Bool,
        featured_local -> Bool,
        url_content_type -> Nullable<Text>,
        alt_text -> Nullable<Text>,
        scheduled_publish_time_at -> Nullable<Timestamptz>,
        newest_comment_time_necro_at -> Nullable<Timestamptz>,
        newest_comment_time_at -> Nullable<Timestamptz>,
        comments -> Int4,
        score -> Int4,
        upvotes -> Int4,
        downvotes -> Int4,
        hot_rank -> Float4,
        hot_rank_active -> Float4,
        controversy_rank -> Float4,
        scaled_rank -> Float4,
        report_count -> Int2,
        unresolved_report_count -> Int2,
        federation_pending -> Bool,
        embed_video_width -> Nullable<Int4>,
        embed_video_height -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PostNotificationsModeEnum;

    post_actions (person_id, post_id) {
        read_at -> Nullable<Timestamptz>,
        read_comments_at -> Nullable<Timestamptz>,
        saved_at -> Nullable<Timestamptz>,
        voted_at -> Nullable<Timestamptz>,
        hidden_at -> Nullable<Timestamptz>,
        person_id -> Int4,
        post_id -> Int4,
        read_comments_amount -> Nullable<Int4>,
        vote_is_upvote -> Nullable<Bool>,
        notifications -> Nullable<PostNotificationsModeEnum>,
    }
}

diesel::table! {
    post_community_tag (post_id, community_tag_id) {
        post_id -> Int4,
        community_tag_id -> Int4,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    post_report (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        #[max_length = 200]
        original_post_name -> Varchar,
        original_post_url -> Nullable<Text>,
        original_post_body -> Nullable<Text>,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        violates_instance_rules -> Bool,
    }
}

diesel::table! {
    private_message (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_id -> Int4,
        content -> Text,
        deleted -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        removed -> Bool,
        deleted_by_recipient -> Bool,
    }
}

diesel::table! {
    private_message_report (id) {
        id -> Int4,
        creator_id -> Int4,
        private_message_id -> Int4,
        original_pm_text -> Text,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    public_case_log (id) {
        id -> Int4,
        case_id -> Int4,
        community_id -> Nullable<Int4>,
        summary -> Text,
        rationale_redacted -> Nullable<Text>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    received_activity (ap_id) {
        ap_id -> Text,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    registration_application (id) {
        id -> Int4,
        local_user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    remote_image (link) {
        link -> Text,
        published_at -> Timestamptz,
    }
}

// v1-federation-inbound-a additions:
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FederationInboxAdminActionEnum;
    use super::sql_types::FederationPeerTrustEnum;

    remote_moderation_label (id) {
        id -> Int4,
        source_instance -> Text,
        actor_url -> Text,
        target_url -> Text,
        label -> Text,
        summary -> Nullable<Text>,
        published_at -> Timestamptz,
        signature -> Text,
        local_case_id -> Nullable<Int4>,
        received_at -> Timestamptz,
        peer_trust_level_at_receipt -> FederationPeerTrustEnum,
        admin_reviewed_at -> Nullable<Timestamptz>,
        admin_action -> FederationInboxAdminActionEnum,
        dismissal_rationale -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FederationInboxAdminActionEnum;
    use super::sql_types::FederationPeerTrustEnum;
    use super::sql_types::SanctionAction;
    use super::sql_types::SanctionScope;

    remote_sanction_notice (id) {
        id -> Int4,
        source_instance -> Text,
        target_url -> Text,
        action -> SanctionAction,
        scope -> SanctionScope,
        summary -> Text,
        published_at -> Timestamptz,
        signature -> Text,
        local_case_id -> Nullable<Int4>,
        received_at -> Timestamptz,
        // v1-federation-inbound-a additions:
        peer_trust_level_at_receipt -> FederationPeerTrustEnum,
        admin_reviewed_at -> Nullable<Timestamptz>,
        admin_action -> FederationInboxAdminActionEnum,
        dismissal_rationale -> Nullable<Text>,
    }
}

diesel::table! {
    report_combined (id) {
        id -> Int4,
        published_at -> Timestamptz,
        post_report_id -> Nullable<Int4>,
        comment_report_id -> Nullable<Int4>,
        private_message_report_id -> Nullable<Int4>,
        community_report_id -> Nullable<Int4>,
        resolved -> Bool,
        item_creator_id -> Nullable<Int4>,
        report_creator_id -> Int4,
        resolver_id -> Nullable<Int4>,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        community_id -> Nullable<Int4>,
        private_message_id -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ReputationDimension;
    use super::sql_types::ReputationEventSourceType;

    reputation_event (id) {
        id -> Int4,
        person_id -> Int4,
        community_id -> Nullable<Int4>,
        dimension -> ReputationDimension,
        delta -> Int4,
        source_case_id -> Nullable<Int4>,
        source_report_id -> Nullable<Int4>,
        reason -> Text,
        created_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
        // v1-RT-r1 additions:
        dedupe_key -> Nullable<Text>,
        source_event_type -> ReputationEventSourceType,
    }
}

diesel::table! {
    reputation_snapshot (id) {
        id -> Int4,
        person_id -> Int4,
        community_id -> Nullable<Int4>,
        reporting_accuracy -> Int4,
        jury_reliability -> Int4,
        participation_consistency -> Int4,
        endorsement_strength -> Int4,
        jury_eligible -> Bool,
        trusted_reporter -> Bool,
        calculated_at -> Timestamptz,
        can_sponsor -> Bool,
    }
}

diesel::table! {
    rule_set_version (id) {
        id -> Int4,
        community_id -> Int4,
        version -> Int4,
        parent_id -> Nullable<Int4>,
        text_sha256 -> Bytea,
        rule_text -> Text,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SanctionScope;
    use super::sql_types::SanctionAction;

    sanction (id) {
        id -> Int4,
        case_id -> Int4,
        scope -> SanctionScope,
        action -> SanctionAction,
        target_person_id -> Nullable<Int4>,
        target_post_id -> Nullable<Int4>,
        target_comment_id -> Nullable<Int4>,
        target_community_id -> Nullable<Int4>,
        starts_at -> Timestamptz,
        ends_at -> Nullable<Timestamptz>,
        active -> Bool,
    }
}

diesel::table! {
    secret (id) {
        id -> Int4,
        jwt_secret -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ActorTypeEnum;

    sent_activity (id) {
        id -> Int8,
        ap_id -> Text,
        data -> Json,
        sensitive -> Bool,
        published_at -> Timestamptz,
        send_inboxes -> Array<Nullable<Text>>,
        send_community_followers_of -> Nullable<Int4>,
        send_all_instances -> Bool,
        actor_type -> ActorTypeEnum,
        actor_apub_id -> Nullable<Text>,
    }
}

diesel::table! {
    site (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Varchar,
        sidebar -> Nullable<Text>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        #[max_length = 150]
        summary -> Nullable<Varchar>,
        #[max_length = 255]
        ap_id -> Varchar,
        last_refreshed_at -> Timestamptz,
        #[max_length = 255]
        inbox_url -> Varchar,
        private_key -> Nullable<Text>,
        public_key -> Text,
        instance_id -> Int4,
        content_warning -> Nullable<Text>,
    }
}

diesel::table! {
    site_language (site_id, language_id) {
        site_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    sponsor_allowlist (id) {
        id -> Int4,
        community_id -> Nullable<Int4>,            // v1-RT-r1: was Int4 (NOT NULL); now nullable
        person_id -> Int4,
        created_at -> Timestamptz,
        // v1-RT-r1 additions:
        added_by_admin_id -> Int4,
        note -> Nullable<Text>,
    }
}

diesel::table! {
    surety (id) {
        id -> Int4,
        sponsor_id -> Int4,
        sponsored_id -> Int4,
        community_id -> Nullable<Int4>,
        created_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tagline (id) {
        id -> Int4,
        content -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(actor_pseudonym -> person (person_id));
diesel::joinable!(appeal -> moderation_case (case_id));
diesel::joinable!(appeal -> person (requester_id));
diesel::joinable!(case_evidence -> moderation_case (case_id));
diesel::joinable!(case_evidence -> person (uploader_id));
diesel::joinable!(comment -> community (community_id));
diesel::joinable!(comment -> language (language_id));
diesel::joinable!(comment -> person (creator_id));
diesel::joinable!(comment -> post (post_id));
diesel::joinable!(comment_actions -> comment (comment_id));
diesel::joinable!(comment_actions -> person (person_id));
diesel::joinable!(comment_report -> comment (comment_id));
diesel::joinable!(community -> instance (instance_id));
diesel::joinable!(community_actions -> community (community_id));
diesel::joinable!(community_language -> community (community_id));
diesel::joinable!(community_language -> language (language_id));
diesel::joinable!(community_report -> community (community_id));
diesel::joinable!(community_tag -> community (community_id));
diesel::joinable!(custom_emoji_keyword -> custom_emoji (custom_emoji_id));
diesel::joinable!(email_verification -> local_user (local_user_id));
diesel::joinable!(endorsement -> community (community_id));
diesel::joinable!(federation_allowlist -> instance (instance_id));
diesel::joinable!(federation_blocklist -> instance (instance_id));
// v1-federation-inbound-a additions:
diesel::joinable!(federation_peer -> instance (instance_id));
diesel::joinable!(federation_queue_state -> instance (instance_id));
diesel::joinable!(governance_config -> person (updated_by));
diesel::joinable!(governance_messaging_config -> person (updated_by));
diesel::joinable!(instance_actions -> instance (instance_id));
diesel::joinable!(instance_actions -> person (person_id));
diesel::joinable!(jury_assignment -> moderation_case (case_id));
diesel::joinable!(jury_assignment -> person (person_id));
diesel::joinable!(jury_constraint_violation_log -> moderation_case (case_id));
diesel::joinable!(jury_pool -> community (community_id));
diesel::joinable!(jury_pool -> person (person_id));
diesel::joinable!(jury_vote -> moderation_case (case_id));
diesel::joinable!(jury_vote -> person (juror_id));
diesel::joinable!(local_image -> person (person_id));
diesel::joinable!(local_image -> post (thumbnail_for_post_id));
diesel::joinable!(local_site -> multi_community (suggested_multi_community_id));
diesel::joinable!(local_site -> person (system_account));
diesel::joinable!(local_site -> site (site_id));
diesel::joinable!(local_site_rate_limit -> local_site (local_site_id));
diesel::joinable!(local_user -> person (person_id));
diesel::joinable!(local_user_keyword_block -> local_user (local_user_id));
diesel::joinable!(local_user_language -> language (language_id));
diesel::joinable!(local_user_language -> local_user (local_user_id));
diesel::joinable!(login_token -> local_user (user_id));
diesel::joinable!(moderation_case -> comment (target_comment_id));
diesel::joinable!(moderation_case -> post (target_post_id));
diesel::joinable!(moderation_case -> rule_set_version (rule_set_version_id));
diesel::joinable!(modlog -> comment (target_comment_id));
diesel::joinable!(modlog -> community (target_community_id));
diesel::joinable!(modlog -> instance (target_instance_id));
diesel::joinable!(modlog -> post (target_post_id));
diesel::joinable!(multi_community -> instance (instance_id));
diesel::joinable!(multi_community -> person (creator_id));
diesel::joinable!(multi_community_entry -> community (community_id));
diesel::joinable!(multi_community_entry -> multi_community (multi_community_id));
diesel::joinable!(multi_community_follow -> multi_community (multi_community_id));
diesel::joinable!(multi_community_follow -> person (person_id));
diesel::joinable!(notification -> comment (comment_id));
diesel::joinable!(notification -> community (community_id));
diesel::joinable!(notification -> instance (instance_id));
diesel::joinable!(notification -> modlog (modlog_id));
diesel::joinable!(notification -> post (post_id));
diesel::joinable!(notification -> private_message (private_message_id));
diesel::joinable!(oauth_account -> local_user (local_user_id));
diesel::joinable!(oauth_account -> oauth_provider (oauth_provider_id));
diesel::joinable!(password_reset_request -> local_user (local_user_id));
diesel::joinable!(person -> instance (instance_id));
diesel::joinable!(person_content_combined -> comment (comment_id));
diesel::joinable!(person_content_combined -> community (community_id));
diesel::joinable!(person_content_combined -> person (creator_id));
diesel::joinable!(person_content_combined -> post (post_id));
diesel::joinable!(person_liked_combined -> comment (comment_id));
diesel::joinable!(person_liked_combined -> community (community_id));
diesel::joinable!(person_liked_combined -> post (post_id));
diesel::joinable!(person_saved_combined -> comment (comment_id));
diesel::joinable!(person_saved_combined -> community (community_id));
diesel::joinable!(person_saved_combined -> post (post_id));
diesel::joinable!(post -> community (community_id));
diesel::joinable!(post -> language (language_id));
diesel::joinable!(post -> person (creator_id));
diesel::joinable!(post_actions -> person (person_id));
diesel::joinable!(post_actions -> post (post_id));
diesel::joinable!(post_community_tag -> community_tag (community_tag_id));
diesel::joinable!(post_community_tag -> post (post_id));
diesel::joinable!(post_report -> post (post_id));
diesel::joinable!(private_message_report -> private_message (private_message_id));
diesel::joinable!(public_case_log -> community (community_id));
diesel::joinable!(public_case_log -> moderation_case (case_id));
diesel::joinable!(registration_application -> local_user (local_user_id));
diesel::joinable!(registration_application -> person (admin_id));
// v1-federation-inbound-a additions:
diesel::joinable!(remote_moderation_label -> moderation_case (local_case_id));
diesel::joinable!(remote_sanction_notice -> moderation_case (local_case_id));
diesel::joinable!(report_combined -> comment (comment_id));
diesel::joinable!(report_combined -> comment_report (comment_report_id));
diesel::joinable!(report_combined -> community (community_id));
diesel::joinable!(report_combined -> community_report (community_report_id));
diesel::joinable!(report_combined -> post (post_id));
diesel::joinable!(report_combined -> post_report (post_report_id));
diesel::joinable!(report_combined -> private_message (private_message_id));
diesel::joinable!(report_combined -> private_message_report (private_message_report_id));
diesel::joinable!(reputation_event -> community (community_id));
diesel::joinable!(reputation_event -> moderation_case (source_case_id));
diesel::joinable!(reputation_event -> person (person_id));
diesel::joinable!(reputation_snapshot -> community (community_id));
diesel::joinable!(reputation_snapshot -> person (person_id));
diesel::joinable!(rule_set_version -> community (community_id));
diesel::joinable!(rule_set_version -> person (created_by));
diesel::joinable!(sanction -> comment (target_comment_id));
diesel::joinable!(sanction -> community (target_community_id));
diesel::joinable!(sanction -> moderation_case (case_id));
diesel::joinable!(sanction -> person (target_person_id));
diesel::joinable!(sanction -> post (target_post_id));
diesel::joinable!(site -> instance (instance_id));
diesel::joinable!(site_language -> language (language_id));
diesel::joinable!(site_language -> site (site_id));
diesel::joinable!(sponsor_allowlist -> community (community_id));
diesel::joinable!(sponsor_allowlist -> person (person_id));
diesel::joinable!(surety -> community (community_id));

diesel::allow_tables_to_appear_in_same_query!(
  actor_pseudonym,
  appeal,
  case_evidence,
  comment,
  comment_actions,
  comment_report,
  community,
  community_actions,
  community_language,
  community_report,
  community_tag,
  email_verification,
  endorsement,
  federation_allowlist,
  federation_blocklist,
  federation_queue_state,
  governance_config,
  governance_messaging_config,
  instance,
  instance_actions,
  jury_assignment,
  jury_constraint_violation_log,
  jury_pool,
  jury_vote,
  language,
  local_image,
  local_site,
  local_site_rate_limit,
  local_user,
  local_user_keyword_block,
  local_user_language,
  login_token,
  moderation_case,
  modlog,
  multi_community,
  multi_community_entry,
  multi_community_follow,
  notification,
  oauth_account,
  oauth_provider,
  password_reset_request,
  person,
  person_content_combined,
  person_liked_combined,
  person_saved_combined,
  post,
  post_actions,
  post_community_tag,
  post_report,
  private_message,
  private_message_report,
  public_case_log,
  registration_application,
  remote_sanction_notice,
  report_combined,
  reputation_event,
  reputation_snapshot,
  rule_set_version,
  sanction,
  site,
  site_language,
  sponsor_allowlist,
  surety,
  person_actions,
  image_details,
  // v1-federation-inbound-a additions:
  federation_inbox_dropped_log,
  federation_inbox_nonce,
  federation_peer,
  remote_moderation_label,
);
diesel::allow_tables_to_appear_in_same_query!(custom_emoji, custom_emoji_keyword,);
