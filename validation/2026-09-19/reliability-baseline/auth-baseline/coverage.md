# Authentication fixture coverage inventory

Source `a826d22a63d9d6889157477a0109e242871796f5`. This is source inspection, not an execution receipt. All rows below are existing tests, not newly passed checks.

| Area | Existing source test |
| --- | --- |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:78` — `failed_writes_reads_and_mismatches_clean_new_accounts_before_install` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:102` — `failed_cleanup_retains_only_account_metadata_and_can_retry_in_another_call` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:131` — `partial_install_restores_prior_binding_and_deletes_only_the_uncommitted_key` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:177` — `failed_verification_never_deletes_an_existing_working_keyring_binding` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:209` — `fresh_partial_install_restores_unbound_configuration_and_removes_the_new_key` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:236` — `interrupted_install_with_bound_account_remains_recoverable` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:277` — `oversized_cleanup_journal_cannot_trigger_deletion` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:297` — `same_uuid_in_legacy_binding_does_not_own_pending_v2_entry` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:337` — `failed_v2_install_preserves_legacy_namespace_and_binding` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:369` — `interrupted_v2_commit_recovers_without_deleting_owned_credential` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:409` — `fresh_oidc_save_read_and_cancelled_commit_failures_leave_no_unowned_token` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:435` — `failed_oidc_cleanup_retries_only_the_recorded_native_format` |
| native transaction rollback | `codex-rs/cli/src/airs_credential_transaction_tests.rs:475` — `cleanup_rejects_symlinks_without_touching_the_referenced_account` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:8` — `verified_restore_preserves_the_active_session_epoch` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:25` — `restore_cannot_reactivate_a_logged_out_or_replaced_session` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:60` — `logout_rejects_an_in_progress_login_before_it_installs` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:74` — `relogin_activates_a_new_epoch_without_resurrecting_old_clients` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:97` — `failed_install_leaves_the_session_revoked` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:118` — `revocation_does_not_wait_for_a_refresh_configuration_lock` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:134` — `short_state_lock_has_a_bounded_wait` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:154` — `a_second_completed_login_cancels_the_first_attempt` |
| authentication epoch and cancellation | `codex-rs/cli/src/airs_auth_lifecycle_tests.rs:170` — `state_lock_does_not_follow_a_symlink` |
| refresh failure classification | `codex-rs/cli/src/airs_oidc_refresh_tests.rs:26` — `definitive_rejection_survives_subsequent_credential_reads` |
| refresh failure classification | `codex-rs/cli/src/airs_oidc_refresh_tests.rs:57` — `uncertain_exchange_keeps_the_tokenless_pending_record` |
| refresh failure classification | `codex-rs/cli/src/airs_oidc_refresh_tests.rs:82` — `returned_generation_is_saved_before_becoming_available` |
| same-identity restore | `codex-rs/cli/src/airs_oidc_restore_tests.rs:4` — `restore_pins_signed_subject_and_configuration_but_not_display_name` |
| namespace isolation | `codex-rs/cli/src/airs_credential_namespace_tests.rs:72` — `conflicting_legacy_cleanup_preserves_shared_root_and_all_metadata` |
| namespace isolation | `codex-rs/cli/src/airs_credential_namespace_tests.rs:106` — `conflicting_legacy_persistence_stops_before_even_unrelated_pending_cleanup` |
| namespace isolation | `codex-rs/cli/src/airs_credential_namespace_tests.rs:153` — `separate_v2_namespace_cleanup_still_preserves_either_legacy_format` |
| logout pending cleanup | `codex-rs/cli/src/airs_logout_cleanup_tests.rs:48` — `deletion_failure_preserves_typed_journal_for_workspace_and_oidc_retries` |
| logout pending cleanup | `codex-rs/cli/src/airs_logout_cleanup_tests.rs:87` — `stale_logout_cannot_delete_another_working_binding` |
| logout pending cleanup | `codex-rs/cli/src/airs_logout_cleanup_tests.rs:106` — `unmarked_state_cannot_resume_destructive_cleanup` |
| logout pending cleanup | `codex-rs/cli/src/airs_logout_cleanup_tests.rs:122` — `malformed_cleanup_and_binding_diagnostics_never_include_file_values` |
| installed doctor read-only and bounds | `scripts/test_airs_doctor.py:36` — `test_environment_pinning_explicit_probe_and_failed_report` |
| installed doctor read-only and bounds | `scripts/test_airs_doctor.py:90` — `test_cancel_slow_diagnostics_then_retry` |
| installed doctor read-only and bounds | `scripts/test_airs_doctor.py:122` — `test_missing_native_service_leaves_binding_and_cleanup_untouched` |
| installed doctor read-only and bounds | `scripts/test_airs_doctor.py:150` — `test_stalled_native_service_probe_is_bounded` |
| installed MCP login lifecycle | `scripts/test_airs_mcp_manager.py:98` — `test_remote_callback_add_verify_logout_remove_and_environment_pinning` |
| installed MCP login lifecycle | `scripts/test_airs_mcp_manager.py:188` — `test_cancel_keeps_connection_and_never_exchanges_or_submits_callback` |
| installed MCP login lifecycle | `scripts/test_airs_mcp_manager.py:199` — `test_desktop_callback_and_failed_discovery_do_not_report_success` |
| installed MCP login lifecycle | `scripts/test_airs_mcp_manager.py:215` — `test_private_adapter_rejects_duplicate_name_without_replacing_it` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:231` — `test_running_client_stays_revoked_after_subprocess_logout_and_relogin` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:334` — `test_plain_terminal_setup_retains_public_settings_after_cancelled_login` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:376` — `test_existing_home_is_reused_without_moving_credentials_or_history` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:448` — `test_oidc_login_without_keyring_explains_recovery_before_network` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:475` — `test_workspace_login_hides_paste_and_restores_terminal_on_cancel` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:513` — `test_workspace_login_rejects_multiline_paste_without_echo_or_binding` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:534` — `test_workspace_login_restores_terminal_when_terminated` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:820` — `test_doctor_json_is_redacted_and_reports_missing_credentials` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:853` — `test_doctor_verify_access_uses_bounded_authenticated_wire_and_selected_route` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:944` — `test_native_status_and_plain_doctor_inspect_configuration_without_store_access` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:984` — `test_doctor_verify_access_denial_preserves_binding_and_redacts_response` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:1085` — `test_named_environments_and_file_credential_without_export` |
| installed onboarding and preserved state | `scripts/test_airs_harness.py:1139` — `test_credential_change_and_repository_destination_override_fail_closed` |

The separate isolated published-binary experiment executed successfully through native save/read and doctor; the backend-divergence hypothesis was disproven. See `../doctor-backend-reproduction.json`.

Pending new tests: preservation and safe composition of primary and cleanup failures; neutral native-store recovery guidance. Owner Ubuntu and attended production SSO/ServiceNow acceptance are deferred by the owner.
