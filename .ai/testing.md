# Testing / CI Tasks

Start with the failing/relevant test file and the code it covers.

Workflow:
1. run the smallest focused test set;
2. inspect only the files needed to explain failures;
3. after implementation changes, run the full relevant CI-equivalent suite.

Rules:
- keep tests offline unless a test explicitly covers update/download behavior;
- use local fixtures for external-source formats;
- do not weaken tests to make a failure disappear;
- preserve regression cases for ambiguity, provenance and licensing gates.

Read CI/workflow files only when the task involves CI behavior.
