"""Owned #48 review + request-bound #49 persistence; no UI/storage inference."""

from alternative_terms import ReviewSession, ChoiceError


class PersistenceError(ValueError):
    """The text choice succeeded; no preference was remembered."""


class PrivateReviewSession:
    def __init__(self, result, usage_context_id):
        # Trusted core result and already validated request context only.
        self._review = ReviewSession(result)
        self._input = result["input"]
        self._source = result["source_locale"]
        self._target = result["target_locale"]
        self._usage = usage_context_id

    def snapshot(self):
        return self._review.snapshot()

    def undo(self, revision):
        # Undo only the current text choice, not previously committed user data.
        return self._review.undo(revision)

    def apply_choice(
        self,
        revision,
        occurrence_id,
        candidate_id,
        intent="use_this_time_only",
        store=None,
    ):
        if intent not in (
            "use_this_time_only",
            "remember_for_this_context",
            "remember_for_all_contexts",
        ):
            raise ChoiceError("Unknown choice intent")
        if intent == "remember_for_this_context" and self._usage is None:
            raise ChoiceError(
                "Select a Usage Context before remembering for this context"
            )
        if intent != "use_this_time_only":
            item = next(
                (
                    item
                    for item in self.snapshot()["occurrences"]
                    if item["occurrence_id"] == occurrence_id
                ),
                None,
            )
            if item is not None and not item.get("rememberable", True):
                raise ChoiceError(
                    "This partial source expansion cannot be safely remembered; use this time only"
                )
        snapshot = self._review.apply_choice(revision, occurrence_id, candidate_id)
        if intent == "use_this_time_only":
            return snapshot
        occurrence = next(
            item
            for item in snapshot["occurrences"]
            if item["occurrence_id"] == occurrence_id
        )
        start, end = occurrence["source_span"]
        try:
            if store is None:
                raise ValueError("No private store")
            store.put_preference(
                "personal",
                self._input[start:end],
                occurrence["expected_text"],
                self._target,
                self._usage if intent == "remember_for_this_context" else None,
                self._source,
            )
        except Exception as error:
            raise PersistenceError(
                "Text choice applied, but nothing was remembered"
            ) from error
        return snapshot
