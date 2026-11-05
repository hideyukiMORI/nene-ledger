"""実データの突き合わせ（ADR 0023・GitHub Issue #26・benchmarks 第 2 節）。

使い方: python3 -I tools/corpus/compare.py <置き場> --binary target/release/nene-ledger
        [--sources docs/quality/corpus/sources.tsv] [--update <版の名前>]

`sources.tsv` の各ファイルを、製品の `--dump` と、正しい文字コードと区切りを与えた Python の `csv.reader`（製品と
独立した参照）で読み、1 レコードずつ比べる。判定は 3 つ:

- `exact`: 一致
- `skipK`: 参照の先頭の K 行（K ≤ 32）を前書きとして飛ばした以外は一致
- `MISMATCH`: 値の読み違い

表の最後の列（`expected_at_<版>`）と判定が違うファイルを並べ、`MISMATCH` が 1 つでもあれば終了 1。
`--update <版の名前>` を付けると、最後の列をこの判定で書き直す（判別を変える PR で使い、変わったファイルを本文に書く）。
置き場に無いファイルは数えるだけで飛ばす（`fetch.py` で取得する）。Python の標準ライブラリだけを使う。
"""

import argparse
import concurrent.futures
import csv
import io
import os
import subprocess
import sys
from collections import Counter

MAX_PREAMBLE = 32

# 1 MiB を超えるセル（引用の中の長い値）も読む。
csv.field_size_limit(1 << 30)


def parse(text, delimiter):
    return list(csv.reader(io.StringIO(text, newline=''), delimiter=delimiter))


def verdict(reference, dumped):
    if reference == dumped:
        return 'exact'
    for skipped in range(1, MAX_PREAMBLE + 1):
        if reference[skipped:] == dumped:
            return f'skip{skipped}'
    return 'MISMATCH'


def judge(row, corpus, binary):
    path = os.path.join(corpus, row['set'], row['file'])
    if not os.path.exists(path):
        return 'absent'
    delimiter = '\t' if row['delimiter'] == '\\t' else row['delimiter']
    try:
        with open(path, 'rb') as source:
            reference = parse(source.read().decode(row['encoding'], 'replace'), delimiter)
    except LookupError:
        return 'unknown-encoding'
    dumped = subprocess.run([binary, '--dump', path], capture_output=True, check=False)
    if dumped.returncode != 0:
        return f'FAIL{dumped.returncode}'
    return verdict(reference, parse(dumped.stdout.decode('utf-8', 'replace'), ','))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('corpus', help='fetch.py で取得した置き場')
    parser.add_argument('--binary', required=True, help='release の nene-ledger')
    parser.add_argument('--sources', default='docs/quality/corpus/sources.tsv')
    parser.add_argument('--update', metavar='VERSION', help='最後の列をこの版の判定で書き直す')
    parser.add_argument('--jobs', type=int, default=os.cpu_count() or 4)
    args = parser.parse_args()
    with open(args.sources, encoding='utf-8', newline='') as table:
        reader = csv.DictReader(table, delimiter='\t')
        fields = reader.fieldnames
        rows = list(reader)
    expected_field = fields[-1]
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
        verdicts = list(pool.map(lambda row: judge(row, args.corpus, args.binary), rows))
    summary = Counter('skip' if v.startswith('skip') else v for v in verdicts)
    print(' · '.join(f'{name} {count}' for name, count in sorted(summary.items())))
    changed = [(row, v) for row, v in zip(rows, verdicts) if v not in ('absent', row[expected_field])]
    for row, v in changed:
        print(f'変わった: {row["set"]}/{row["file"]} {row[expected_field]} -> {v}')
    print(f'表（{expected_field}）と違うファイル {len(changed)}')
    if args.update:
        new_field = f'expected_at_{args.update}'
        with open(args.sources, 'w', encoding='utf-8', newline='') as table:
            writer = csv.writer(table, delimiter='\t', lineterminator='\n')
            writer.writerow(fields[:-1] + [new_field])
            for row, v in zip(rows, verdicts):
                kept = row[expected_field] if v == 'absent' else v
                writer.writerow([row[name] for name in fields[:-1]] + [kept])
        print(f'{args.sources} の最後の列を {new_field} で書き直した')
    return 1 if summary.get('MISMATCH') else 0


if __name__ == '__main__':
    sys.exit(main())
