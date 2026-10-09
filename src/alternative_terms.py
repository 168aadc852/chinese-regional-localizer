"""Reference for shared occurrence choices; no localization reruns or persistence."""

from copy import deepcopy


class ChoiceError(ValueError):
    """The caller must refresh/review instead of guessing a different position."""


def make_choice(ranked, source_text, source_span, stage_span, output_span, current):
    best = {}
    for target, level, priority in ranked:
        rank = (level, -priority)
        if target not in best or rank < best[target]:
            best[target] = rank
    ordered = sorted(best, key=lambda target: (*best[target], target))
    top = best[ordered[0]]
    tied = sum(rank == top for rank in best.values()) > 1
    candidates = [
        {
            "candidate_id": f"candidate-{i + 1}",
            "target_text": target,
            "status": ("needs_decision" if tied else "recommended")
            if best[target] == top
            else "also_valid",
        }
        for i, target in enumerate(ordered)
    ]
    return {
        "occurrence_id": "",
        "state": "needs_decision" if tied else "recommended",
        "source_text": source_text,
        "source_span": list(source_span),
        "stage_input_span": list(stage_span),
        "output_span": list(output_span),
        "expected_text": current,
        "selected_candidate_id": None if tied else "candidate-1",
        "candidates": candidates,
    }


def assign_choice_ids(events):
    choices = [event["choice"] for event in events if "choice" in event]
    choices.sort(key=lambda choice: tuple(choice["output_span"]))
    for i, choice in enumerate(choices):
        occurrence = f"occurrence-{i + 1}"
        selected = choice["selected_candidate_id"]
        for j, candidate in enumerate(choice["candidates"]):
            previous = candidate["candidate_id"]
            candidate["candidate_id"] = f"{occurrence}/candidate-{j + 1}"
            if previous == selected:
                choice["selected_candidate_id"] = candidate["candidate_id"]
        choice["occurrence_id"] = occurrence


class ReviewSession:
    """In-memory snapshot from core results; getters return copies, not live state.

    Construct only from trusted core results, never arbitrary frontend candidates.
    A future client retains this state in the core, not in editable UI metadata.
    """

    def __init__(self, result):
        self._text = result["output"]
        self._occurrences = deepcopy(
            [event["choice"] for event in result["changes"] if "choice" in event]
        )
        self._occurrences.sort(key=lambda item: tuple(item["output_span"]))
        self._revision = 0
        self._undo = []
        for item in self._occurrences:
            if (
                len(item["source_span"]) != 2
                or any(type(value) is not int for value in item["source_span"])
                or not 0
                <= item["source_span"][0]
                < item["source_span"][1]
                <= len(result["input"])
                or not 0 <= item["stage_input_span"][0] < item["stage_input_span"][1]
            ):
                raise ChoiceError("Invalid source tracking")
        self._validate()

    def snapshot(self):
        return {
            "text": self._text,
            "revision": self._revision,
            "occurrences": deepcopy(self._occurrences),
        }

    def _validate(self):
        end = 0
        ids = set()
        for occurrence in self._occurrences:
            span = occurrence["output_span"]
            if (
                len(span) != 2
                or any(type(value) is not int for value in span)
                or not end <= span[0] <= span[1] <= len(self._text)
                or self._text[span[0] : span[1]] != occurrence["expected_text"]
                or not occurrence["occurrence_id"]
                or occurrence["occurrence_id"] in ids
            ):
                raise ChoiceError(
                    "Occurrence tracking is stale or invalid; refresh review"
                )
            ids.add(occurrence["occurrence_id"])
            end = span[1]
            candidates = occurrence["candidates"]
            candidate_ids = [item["candidate_id"] for item in candidates]
            if (
                not candidates
                or len(set(candidate_ids)) != len(candidate_ids)
                or len({item["target_text"] for item in candidates}) != len(candidates)
            ):
                raise ChoiceError("Invalid frozen candidate set")
            if any(
                not item["candidate_id"].startswith(occurrence["occurrence_id"] + "/")
                or item["status"] not in ("recommended", "also_valid", "needs_decision")
                for item in candidates
            ):
                raise ChoiceError("Invalid frozen candidate set")
            selected = occurrence["selected_candidate_id"]
            if occurrence["state"] == "needs_decision":
                if (
                    selected is not None
                    or occurrence["expected_text"] != occurrence["source_text"]
                ):
                    raise ChoiceError("Invalid unresolved occurrence")
            elif occurrence["state"] in ("recommended", "chosen"):
                match = next(
                    (item for item in candidates if item["candidate_id"] == selected),
                    None,
                )
                if match is None or match["target_text"] != occurrence["expected_text"]:
                    raise ChoiceError("Invalid selected occurrence")
            else:
                raise ChoiceError("Invalid occurrence state")

    def _check(self, revision):
        if type(revision) is not int or revision != self._revision:
            raise ChoiceError("Stale review revision; refresh review")
        self._validate()
        if self._revision >= (1 << 64) - 1:
            raise ChoiceError("Review revision exhausted; start a new review")

    def _replace(self, index, text):
        occurrence = self._occurrences[index]
        start, end = occurrence["output_span"]
        delta = len(text) - (end - start)
        self._text = self._text[:start] + text + self._text[end:]
        occurrence["output_span"] = [start, start + len(text)]
        occurrence["expected_text"] = text
        for later in self._occurrences[index + 1 :]:
            later["output_span"] = [value + delta for value in later["output_span"]]

    def apply_choice(
        self, revision, occurrence_id, candidate_id, intent="use_this_time_only"
    ):
        if intent != "use_this_time_only":
            raise ChoiceError("Choice intent is not implemented; nothing was saved")
        self._check(revision)
        index = next(
            (
                i
                for i, item in enumerate(self._occurrences)
                if item["occurrence_id"] == occurrence_id
            ),
            None,
        )
        if index is None:
            raise ChoiceError("Unknown occurrence; refresh review")
        occurrence = self._occurrences[index]
        candidate = next(
            (
                item
                for item in occurrence["candidates"]
                if item["candidate_id"] == candidate_id
            ),
            None,
        )
        if candidate is None:
            raise ChoiceError("Candidate does not belong to this occurrence")
        previous = deepcopy(occurrence)
        self._replace(index, candidate["target_text"])
        occurrence["selected_candidate_id"] = candidate_id
        occurrence["state"] = "chosen"
        self._undo.append((index, previous, deepcopy(occurrence)))
        self._revision += 1
        return self.snapshot()

    def undo(self, revision):
        self._check(revision)
        if not self._undo:
            raise ChoiceError("No one-time choice to undo")
        index, previous, expected = self._undo[-1]
        if self._occurrences[index] != expected:
            raise ChoiceError("Undo state is stale; refresh review")
        self._replace(index, previous["expected_text"])
        self._occurrences[index] = deepcopy(previous)
        self._undo.pop()
        self._revision += 1
        return self.snapshot()
