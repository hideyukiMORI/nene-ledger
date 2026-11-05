"""実データの突き合わせのファイルを取得する（ADR 0023・GitHub Issue #26）。

使い方: python3 -I tools/corpus/fetch.py <置き場> [--sources docs/quality/corpus/sources.tsv]

`sources.tsv` の各行の URL を `<置き場>/<set>/<file>` に保存する（既にあれば取り直さない）。
URL の無い行（`-`）は飛ばす。大きさが表と違えば知らせる（出どころが更新された）。Python の標準ライブラリだけを使う。
"""

import argparse
import csv
import os
import sys
import urllib.request

USER_AGENT = 'nene-ledger-corpus-fetch/1'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('corpus', help='ファイルを置く場所（リポジトリの外）')
    parser.add_argument('--sources', default='docs/quality/corpus/sources.tsv')
    args = parser.parse_args()
    fetched = skipped = missing = changed = failed = 0
    with open(args.sources, encoding='utf-8', newline='') as table:
        for row in csv.DictReader(table, delimiter='\t'):
            target = os.path.join(args.corpus, row['set'], row['file'])
            if row['url'] == '-':
                missing += 1
                continue
            if os.path.exists(target):
                skipped += 1
            else:
                os.makedirs(os.path.dirname(target), exist_ok=True)
                request = urllib.request.Request(row['url'], headers={'User-Agent': USER_AGENT})
                try:
                    with urllib.request.urlopen(request, timeout=120) as response, open(target, 'wb') as out:
                        out.write(response.read())
                    fetched += 1
                except OSError as error:
                    failed += 1
                    print(f'取得できなかった: {row["set"]}/{row["file"]} {row["url"]} ({error})', file=sys.stderr)
                    continue
            size = os.path.getsize(target)
            if str(size) != row['bytes']:
                changed += 1
                print(f'大きさが表と違う: {row["set"]}/{row["file"]} 表 {row["bytes"]} 今 {size}', file=sys.stderr)
    print(f'取得 {fetched}・既にある {skipped}・URL 無し {missing}・大きさが違う {changed}・失敗 {failed}')
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
