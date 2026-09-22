use super::AirsRecoveryState;
use super::RecoveryOperation;
use pretty_assertions::assert_eq;

#[test]
fn sign_in_cancellation_does_not_take_ownership_of_other_recovery_operations() {
    for operation in [
        RecoveryOperation::Doctor,
        RecoveryOperation::Mcp,
        RecoveryOperation::TypeSafe,
        RecoveryOperation::CompanySignIn,
    ] {
        let mut state = AirsRecoveryState::default();
        assert_eq!(state.company_sign_in_attempt(), None);
        let (first, _) = state.begin(RecoveryOperation::CompanySignIn).unwrap();
        assert_eq!(state.company_sign_in_attempt(), Some(first));
        assert!(state.finish(first));
        assert_eq!(state.company_sign_in_attempt(), None);

        let (second, cancellation) = state.begin(operation).unwrap();
        // An attempted concurrent login must not change who owns the active work.
        assert!(state.begin(RecoveryOperation::CompanySignIn).is_none());
        assert_eq!(
            state.company_sign_in_attempt(),
            (operation == RecoveryOperation::CompanySignIn).then_some(second)
        );
        assert!(!state.cancel_attempt(first));
        assert!(!state.finish(first));
        assert!(!cancellation.is_cancelled());
        assert!(state.is_current(second));
        assert!(state.cancel_attempt(second));
        assert!(cancellation.is_cancelled());
        assert_eq!(state.company_sign_in_attempt(), None);
        assert!(!state.finish(second));
        let (retry, _) = state.begin(RecoveryOperation::CompanySignIn).unwrap();
        assert_eq!(state.company_sign_in_attempt(), Some(retry));
    }
}
