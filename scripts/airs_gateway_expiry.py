"""Exercise active use without changing the gateway token's real expiry."""

import time

WAIT_SECONDS = 3605
ACTIVITY_INTERVAL_SECONDS = 600
QUIET_BEFORE_EXPIRY_SECONDS = 600


def wait_with_active_session(
    expires_at_ms,
    activity,
    *,
    monotonic=time.monotonic,
    wall_clock=time.time,
    sleep=time.sleep,
):
    """Keep ordinary SSO grants active, then leave the frontend expiry untouched.

    The deployment's refresh grants have a thirty-minute idle limit. A read
    every ten minutes exercises active use; the final quiet window ensures
    that the scheduled concurrent processes encounter an expired frontend token.
    The caller checks that each earlier read leaves that token unchanged.
    """
    started = monotonic()
    deadline = started + WAIT_SECONDS
    next_activity = started + ACTIVITY_INTERVAL_SECONDS
    count = 0
    while monotonic() < deadline:
        now = monotonic()
        if now >= next_activity:
            if expires_at_ms / 1000 - wall_clock() > QUIET_BEFORE_EXPIRY_SECONDS:
                count += 1
                activity(count)
            next_activity += ACTIVITY_INTERVAL_SECONDS
            continue
        sleep(min(30, deadline - now, next_activity - now))
    return count
