# THUOCL (THU Open Chinese Lexicon)

Status: **approved_with_conditions**

## Intended use
Mainland Chinese vocabulary coverage, domain lexicons, word-frequency signals, segmentation support and candidate detection for regional terminology.

## Official source
Repository: https://github.com/thunlp/THUOCL
Publisher: THUNLP / Tsinghua University NLP and Social Computing Lab

## Evidence reviewed
- Repository root contains an MIT License file (copyright THUNLP).
- README describes THUOCL as an open Chinese lexicon.
- README explicitly states that THUOCL is freely open to universities, research institutes, enterprises, institutions and individuals and may be used for research and commercial purposes.
- README requests citation when research papers or scientific outputs are based on THUOCL.
- README explains that the word lists were curated from social tags, search hot terms, input-method lexicons and frequency corpora including CSDN, Sina News and Sogou corpora.

## Rights review
For the repository distribution itself:
- Commercial use: **Yes**
- Use: **Yes**
- Copy/modify/distribute: **MIT repository licence indicates yes for the repository distribution, subject to preservation of the MIT notice**
- Attribution/licence notice: **Preserve MIT copyright/licence notice**
- Research citation: **Requested by the project README for research/scientific outputs**
- Share-alike/copyleft: **No**

## Important provenance limitation
The word lists were produced from third-party websites/corpora/input-method resources. The project’s open-source/commercial-use statement and root MIT licence support reuse of the published THUOCL package, but they do not transfer rights in the underlying raw third-party corpora.

Therefore:
- use only the published THUOCL word-list/frequency files;
- do not redistribute or reconstruct source corpora such as CSDN/Sina/Sogou from THUOCL provenance statements;
- retain source-category/provenance metadata where practical;
- treat the DF values as THUOCL-derived frequency signals, not as rights to the original corpus content.

## Packaging decision
Approved for the attribution/permissive candidate layer with MIT notice preservation and clear provenance.

Suitable uses:
- Chinese word detection;
- domain classification hints;
- Mainland vocabulary candidates;
- frequency ranking.

Not suitable as a sole authority for HK/TW localisation or as proof that a term is officially preferred in Mainland China.

## Update method
Track the official GitHub repository. Record commit/release used because the project is updated by repository changes rather than a formal versioned release process.

Last reviewed: 2026-10-06
