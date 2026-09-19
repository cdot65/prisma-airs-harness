test_cancel_slow_diagnostics_then_retry (test_airs_doctor.SessionDoctor.test_cancel_slow_diagnostics_then_retry) ... ok
test_environment_pinning_explicit_probe_and_failed_report (test_airs_doctor.SessionDoctor.test_environment_pinning_explicit_probe_and_failed_report) ... ok
test_missing_native_service_leaves_binding_and_cleanup_untouched (test_airs_doctor.SessionDoctor.test_missing_native_service_leaves_binding_and_cleanup_untouched) ... ok
test_redacted_report_preview_copy_and_save_reuse_one_inspection (test_airs_doctor.SessionDoctor.test_redacted_report_preview_copy_and_save_reuse_one_inspection) ... FAIL
test_stalled_native_service_probe_is_bounded (test_airs_doctor.SessionDoctor.test_stalled_native_service_probe_is_bounded) ... ok

======================================================================
FAIL: test_redacted_report_preview_copy_and_save_reuse_one_inspection (test_airs_doctor.SessionDoctor.test_redacted_report_preview_copy_and_save_reuse_one_inspection)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "/home/cdot/development/cdot65/airs-post011-reliability-20260919/scripts/test_airs_doctor.py", line 163, in test_redacted_report_preview_copy_and_save_reuse_one_inspection
    terminal.wait_for(b"AIRS diagnostic report", offset)
    ~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/home/cdot/development/cdot65/airs-post011-reliability-20260919/scripts/airs_harness_pty.py", line 109, in wait_for
    self.wait_until(lambda: marker in self.transcript[offset:], timeout)
    ~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/home/cdot/development/cdot65/airs-post011-reliability-20260919/scripts/airs_harness_pty.py", line 106, in wait_until
    raise AssertionError(self.transcript.decode(errors="replace")[-5000:])
AssertionError: ion and draft stay here. Esc cancels.[17;1H[22m [18;1H[1m[38;5;6;49m› 1. Cancel  Stop this check; keep your sign-in and draft.[19;1H[22m[39;49m [20;1H  [2mPress enter to confirm or esc to go back                                                                              [39m[49m[0m[?25l[?2026l[?2026h[39m[49m[0m[?25l[?2026l[?2026h[39m[49m[0m[?25l[?2026l[?2026h[13;1H[J[13;2H[0m[49m[K[14;2H[0m[49m[K[15;27H[0m[49m[K[16;71H[0m[49m[K[17;2H[0m[49m[K[18;92H[0m[49m[K[19;109H[0m[49m[K[20;118H[0m[49m[K[21;64H[0m[49m[K[22;110H[0m[49m[K[23;116H[0m[49m[K[24;69H[0m[49m[K[25;107H[0m[49m[K[26;72H[0m[49m[K[27;2H[0m[49m[K[13;1H [14;1H [15;1H  [1mConnection health · work[16;1H[22m  [2mWorkspace credential reference. No conversation or tool is replayed.[17;1H[22m [18;1H[1m[38;5;6;49m› 1. Refresh diagnostics               Read configuration and health; no inference request.[19;1H[22m[39;49m  2. Verify gateway access             [2mConfirm a minimal inference request; no conversation, files or tools.[20;1H[22m  3. Credential recovery               [2mTo replace a workspace key, exit and run airs --environment NAME login for the[21;1H[22m                                       [2menvironment shown above.[22;1H[22m  4. Gateway access · Not verified     [2mA saved credential and health response do not verify inference access.[23;1H[22m  5. gateway health · Needs attention  [2mHealth probe failed; check gateway DNS, TLS and its API-root health endpoint[24;1H[22m  6. credential cleanup · OK           [2mNo pending credential cleanup[25;1H[22m  7. credential configuration · OK     [2mReference credential available locally; gateway access not checked.[26;1H[22m  8. configuration · OK                [2mhttp://127.0.0.1:40523/prefix/v1[27;1H[22m [28;1H  [2mPress enter to confirm or esc to go back                                                                              [39m[49m[0m[?25l[?2026l[?2026h[13;1H[J[13;2H[0m[49m[K[14;2H[0m[49m[K[15;20H[0m[49m[K[16;115H[0m[49m[K[17;24H[0m[49m[K[18;2H[0m[49m[K[19;65H[0m[49m[K[20;93H[0m[49m[K[21;84H[0m[49m[K[22;74H[0m[49m[K[23;2H[0m[49m[K[13;1H [14;1H [15;1H  [1mDiagnostic report[16;1H[22m  [2mVersion, platform, authentication method and check results. Names, addresses, paths and raw details are omitted.[17;1H[22m  [2mNothing is uploaded. [18;1H[22m [19;1H[1m[38;5;6;49m› 1. Preview report     Read the exact report; Esc returns here.[20;1H[22m[39;49m  2. Copy report        [2mSend plain text to your clipboard; terminal support may be required.[21;1H[22m  3. Save local report  [2mCreate a private text file in this environment's directory.[22;1H[22m  4. Close              [2mReturn to your conversation; preserve your draft.[23;1H[22m [24;1H  [2mPress enter to confirm or esc to go back                                                                              [39m[49m[0m[?25l[?2026l[?2026h[1;12r[12S[r[13;1H[J[1;1H[2m/ Diagnostic report / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / / [2;1H[22mAIRS[2;6Hdiagnostic[2;17Hreport[2;24Hv1[3;1HPrisma[3;8HAIRS[3;13HHarness[3;21H0.1.1[4;1HPlatform:[4;11Hlinux[4;17H/[4;19Hx86_64[5;1HAuthentication:[5;17HWorkspace[5;27Hcredential[5;38Hreference[7;1HSnapshot[7;10Hof[7;13Hthe[7;17Hlast[7;22H/doctor[7;30Hcheck;[7;37Hno[7;40Hnew[7;44Hrequests[7;53Hwere[7;58Hmade.[8;1HNames,[8;8Haddresses,[8;19Hpaths,[8;26Hraw[8;30Hdetails[8;38Hand[8;42Hconversation[8;55Hcontent[8;63Hare[8;67Homitted.[10;1Hcredential_cleanup:[10;21HPASS[11;1Hlocal_tools:[11;14HPASS[12;1Hlinux_user_namespaces:[12;24HPASS[13;1Hconfiguration:[13;16HPASS[14;1Hcredential_configuration:[14;27HPASS[15;1Hcapabilities:[15;15HPASS[16;1Hgateway_health:[16;17HNeeds[16;23Hattention[17;3HRecovery:[17;13HCheck[17;19Hgateway[17;27HDNS,[17;32HTLS[17;36Hand[17;40Hthe[17;44HAPI-root[17;53Hhealth[17;60Hendpoint.[18;1Hmcp_configuration:[18;20HPASS[19;1Hgateway_access:[19;17HNot[19;21Hverified[21;1HCredential-service[21;20Hhealth[21;27Hdoes[21;32Hnot[21;36Hverify[21;43Ha[21;45Hsaved[21;51Hcredential.[22;1HGateway[22;9Hhealth[22;16Hdoes[22;21Hnot[22;25Hverify[22;32Hinference[22;42Haccess.[22;50HMCP[22;54Hdiscovery[22;64Hdoes[22;69Hnot[22;73Hverify[22;80Ha[22;82Hcompleted[22;92Htool[22;97Hcall.[23;1HNo[23;4Hcredentials,[23;17Hconfiguration,[23;32Hraw[23;36Hlogs[23;41Hor[23;44Htool[23;49Hresults[23;57Hare[23;61Hincluded.[24;1H~[25;1H~[26;1H~[27;1H~[28;1H~[29;1H~[30;1H~[31;1H~[32;1H~[33;1H~[34;1H~[35;1H~[36;1H~[37;1H[2m───────────────────────────────────────────────────────────────────────────────────────────────────────────────── 100% ─[38;1H ↑/↓ to scroll   pgup/pgdn to page   home/end to jump[39;1H q close[39m[49m[0m[?25l[?2026l

----------------------------------------------------------------------
Ran 5 tests in 43.859s

FAILED (failures=1)
Traceback (most recent call last):
  File "/home/cdot/development/cdot65/airs-post011-reliability-20260919/scripts/airs_release_unittest.py", line 56, in <module>
    main()
    ~~~~^^
  File "/home/cdot/development/cdot65/airs-post011-reliability-20260919/scripts/airs_release_unittest.py", line 52, in main
    run_suite(args.scripts, args.pattern, args.receipt)
    ~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/home/cdot/development/cdot65/airs-post011-reliability-20260919/scripts/airs_release_unittest.py", line 39, in run_suite
    require(
    ~~~~~~~^
        summary["passed"],
        ^^^^^^^^^^^^^^^^^^
        "Installed fixture suite failed, was empty or entirely skipped",
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    )
    ^
  File "/home/cdot/development/cdot65/airs-post011-reliability-20260919/scripts/airs_test_release_spec.py", line 33, in require
    raise ValueError(message)
ValueError: Installed fixture suite failed, was empty or entirely skipped
