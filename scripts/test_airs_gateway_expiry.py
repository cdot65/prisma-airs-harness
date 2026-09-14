import unittest

from airs_gateway_expiry import wait_with_active_session


class Clock:
    def __init__(self):
        self.now = 0
        self.sleeps = []

    def sleep(self, seconds):
        self.sleeps.append(seconds)
        self.now += seconds


class GatewayExpiryTests(unittest.TestCase):
    def exercise(self, activity_duration):
        clock = Clock()
        reads = []
        # Login and initial tools have already consumed part of the token life.
        expires_at = 3470

        def read(index):
            reads.append(clock.now)
            clock.now += activity_duration

        count = wait_with_active_session(
            expires_at * 1000,
            read,
            monotonic=lambda: clock.now,
            wall_clock=lambda: clock.now,
            sleep=clock.sleep,
        )
        self.assertGreaterEqual(clock.now, 3605)
        self.assertGreater(clock.now, expires_at)
        self.assertEqual(count, len(reads))
        self.assertGreaterEqual(count, 3)
        self.assertTrue(all(expires_at - when > 600 for when in reads))
        activity = [0, *reads, clock.now]
        self.assertTrue(all(b - a < 1800 for a, b in zip(activity, activity[1:])))
        self.assertTrue(all(0 < seconds <= 30 for seconds in clock.sleeps))

    def test_active_reads_preserve_actual_expiry_and_idle_policy(self):
        self.exercise(0)

    def test_slow_reads_do_not_shorten_the_expiry_wait(self):
        self.exercise(240)

    def test_failed_activity_stops_acceptance(self):
        clock = Clock()

        def failed(index):
            raise RuntimeError("The actual native workflow failed")

        with self.assertRaisesRegex(RuntimeError, "actual native workflow"):
            wait_with_active_session(
                3600000,
                failed,
                monotonic=lambda: clock.now,
                wall_clock=lambda: clock.now,
                sleep=clock.sleep,
            )


if __name__ == "__main__":
    unittest.main()
