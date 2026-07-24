"""Native module smoke tests (requires just build-auth-py)."""

from __future__ import annotations

import unittest

from nibli_auth import (
    allowed_fields,
    can,
    can_any,
    load_policy,
    policy_version,
    reset_thread_auth,
)


class AuthCoreTests(unittest.TestCase):
    def setUp(self) -> None:
        reset_thread_auth()

    def test_policy_version(self) -> None:
        self.assertEqual(policy_version(), "0.1.0")

    def test_owner_can_update(self) -> None:
        d = can("Alice", "update", "Doc1", "owns(Alice, Doc1).")
        self.assertTrue(d.allowed, d)

    def test_stranger_denied(self) -> None:
        d = can("Bob", "update", "Doc1", "owns(Alice, Doc1).")
        self.assertFalse(d.allowed, d)

    def test_admin(self) -> None:
        ctx = 'has_role(Carol, "admin").\nresource(Doc1).'
        d = can("Carol", "update", "Doc1", ctx)
        self.assertTrue(d.allowed, d)

    def test_allowed_fields_owner(self) -> None:
        fields = allowed_fields(
            "Alice", "read", "Doc1", ["title", "body"], "owns(Alice, Doc1)."
        )
        self.assertIn("title", fields)
        self.assertIn("body", fields)

    def test_can_any(self) -> None:
        ctx = 'owns(Alice, Doc1).\nin_tenant(Alice, "acme").\nresource_tenant(Doc2, "acme").'
        results = can_any("Alice", "read", ["Doc1", "Doc2", "Doc3"], ctx)
        self.assertEqual(
            results,
            [("Doc1", True), ("Doc2", True), ("Doc3", False)],
        )

    def test_grant(self) -> None:
        ctx = 'grant(Alice, "edit", Doc1).'
        d = can("Alice", "edit", "Doc1", ctx)
        self.assertTrue(d.allowed, d)
        d_wrong = can("Alice", "delete", "Doc1", ctx)
        self.assertFalse(d_wrong.allowed, d_wrong)

    def test_load_policy(self) -> None:
        version = load_policy('all $a, $r: has_role($a, "super") & resource($r) -> authorized($a, "all", $r).')
        self.assertEqual(version, "0.1.0")
        ctx = 'has_role(Alice, "super").\nresource(Doc100).'
        d = can("Alice", "all", "Doc100", ctx)
        self.assertTrue(d.allowed, d)


if __name__ == "__main__":
    unittest.main()

