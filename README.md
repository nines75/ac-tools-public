# ac-tools-rs

> [!WARNING]
> このソフトウェアは2024年4月ごろに作成し数か月のみ使用していたもので、現在の環境でのテストは行われていません  
> そのため使用はおすすめしません

## これはなに？

ojと併用することを前提とした、C++での競技プログラミング支援ツール

## なぜojだけじゃ駄目なの？

たとえばojだと`テストケースのダウンロード => テスト => 提出`の一連の流れを行うのに以下のようなコマンドが必要です

```sh
oj d https://atcoder.jp/contests/agc001/tasks/agc001_a
g++ main.cpp
oj t
oj s
```

この際、全てのテストケースにパスしたか手動で確認して提出する必要があります

これが、ac-tools-rsだと以下のコマンドのみで行えます(`ac gen`で生成したコンテスト環境内の場合)

```sh
a -a
```

こちらでは、全てのテストケースにパスした場合のみ提出されます  
テストケースはキャッシュされるため余計な通信は発生しません

他にも、C++だとデバッグ用のコンパイルオプションを有効化するとコンパイル速度や実行速度が低下します  
そのため、ベースの実行コマンドとデバッグ用のコンパイルオプションを分離して管理し、切り替えられるようにしています  
例えば先ほどの例で、デバッグ用のコンパイルオプションを有効にしてテストしたい場合以下のようにして行えます

```sh
a -ad
```

テストケースのダウンロードやテストはojをそのまま使用していますが、AtCoderへの提出だけは新たに実装したものがデフォルトとなっています  
これは、ojの提出を利用すると頻繁に429エラーが発生するためです

## なにができるの？

大まかに以下のように分類できます

- ojの機能をより簡単に利用できるようにしたもの
    - テスト
    - デバッグ用コマンド/ヘッダーへの切り替え
- ojの機能を置き換えるもの
    - 提出(AtCoder以外はojを使用)
- それ以外の機能
    - 一括プリコンパイル
    - テストケース生成支援ツール(ojでダウンロードできないもの)
    - gdbコマンドによるデバッグの簡略化

## インストール

1. [oj](https://github.com/online-judge-tools/oj)をインストール
2. ac-tools-rsをインストール
    ```sh
    git clone https://github.com/nines75/ac-tools-rs.git
    cd ac-tools-rs
    cargo install --path .
    ```
3. シェルのconfigファイルに初期化コマンドを書く
    <details>
    <summary>bash</summary>

    ```bash
    eval "$(ac init)"
    ```

    </details>
    <details>
    <summary>fish</summary>

    ```fish
    ac init --fish | source
    ```

    </details>

> [!WARNING]
> 初期化コマンドは必ずconfigファイルの末尾に記述してください

4. 環境変数を設定

    - `AC_BASE_PATH`
        - ベースディレクトリとして使用するパス
        - チルダ（`~`）は展開されます
    - `AC_CONTEST_NAME`(オプション)
        - 検索するatcoder-problemsのバーチャルコンテストの名前（部分一致）
        - スペース区切りで複数のコンテスト名を設定できます
    - `AC_USE_OJ` (オプション)
        - 常に提出にojを使用する
        - 設定する値は任意です

5. コンパイルに使用するコマンドを指定  
   これらのファイルは`[AC_BASE_PATH]/setting`に配置する必要があります
    - [`cpp.txt`](./example/setting/cpp.txt)
        - C++コードをコンパイルする際に使用するコマンド
    - [`cpp_options.txt`](./example/setting/cpp_options.txt)
        - `--debug`オプションが有効になっているとき、基本コマンド(`cpp.txt`)に加えて付与されるコンパイルオプション
    - [`cpp_header.txt`](./example/setting/cpp_header.txt)
        - ヘッダーをプリコンパイルする際に使用するコマンド
6. コアダンプの出力先を変更(オプション)  
   `ac debug`を使う場合はこの設定が必要です

    出力先: `[AC_BASE_PATH]/tmp`

7. プリンパイル用ヘッダを配置(オプション)  
   `ac precompile`を使う場合はこの設定が必要です

    配置先: `[AC_BASE_PATH]/library/header`

## 機能

### テストコマンドに共通する引数

- `--debug`(`-d`)
    - コンパイル時に使用するコマンドに、`cpp.txt`のものに`cpp_options.txt`の引数を加える
- `--auto`(`-a`)
    - テストにパスしたとき、自動で提出する

### `ac`

#### **`ac gen(g)`**

コンテスト環境を生成

#### **`ac test(t) [options] <URL/コンテストID> <問題ID>`**

指定した問題をテスト

デフォルトでは第一引数をURLと解釈します  
URLで指定する場合、問題IDは不要です

> [!WARNING]
> AtCoder以外のコンテストサイトは、URLを用いた問題の指定のみ利用できます

options

- `--custom`(`-c`)
    - AtCoderのコンテストIDと問題IDから問題を指定してテスト

#### **`ac precompile(p)`**

対象のディレクトリに存在するヘッダーをプリコンパイルする

#### **`ac testcase(m)`**

テストケースを作成

> [!WARNING]
> 作成できるのは、ojがテストケースのダウンロードに対応していないサイト(Codeforces)用のものだけです

#### **`ac submit(s) [options]`**

直近にテストしたファイルを提出

options

- `--oj`(`-o`)
    - 提出にojを使う

#### **`ac debug(d) <ファイル名>`**

C++コードとコアダンプをもとにgdbコマンドを実行

#### **`ac nodebug(n) <ファイル名>`**

`cpp.txt`のコマンドでC++コードを実行

稀に、コンパイルオプションを追加しているときだけエラーをはく場合や実行が極端に遅くなる場合があるため、そのような場合にテストするためのコマンドです

### `abc`/`arc`/`agc`

**`abc [options] <コンテスト番号> <問題ID>`**

`ac test`コマンドのAtCoder用エイリアス

> [!WARNING]
> 他のコマンドと異なり、指定するのはコンテストIDではなくコンテスト番号です  
> コンテストIDが`abc123`であれば、コンテスト番号は`123`となります

### `yk`

**`yk [options] <問題ID>`**

`ac test`コマンドのyukicoder用エイリアス

問題IDはURLから抽出できます。
例:`https://yukicoder.me/problems/no/[問題ID]`

### `cf`

**`cf [options] <コンテストID> <問題ID>`**

`ac test`コマンドのCodeforces用エイリアス

### `a`~`h`

**`a [options]`**

`ac test`コマンドのコンテスト環境用エイリアス

`ac gen`で作成したコンテスト環境内で使用することを想定しています

コンテストIDはディレクトリから、問題IDはコマンド自体から抽出されます
