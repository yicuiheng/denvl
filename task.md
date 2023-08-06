- [ ] 一時的に run-experimental コマンドを追加
- [ ] run-experimental コマンド実行時にはクライアントは単に個別の名前付きパイプを作って、その上でやり取りして終わる
- [ ] 修正内容を確認

---

ファイル置き場

- 共通の名前付きパイプ。サーバプロセスが動いている間は存在して、サーバプロセスの終了とともに削除される。クライアントの uuid を取得するために使われる。

  - /tmp/denvl/pipes/common.client2server.pipe
  - /tmp/denvl/pipes/common.server2client.pipe

- クライアントごとのパイプ。クライアントがサーバに依頼した時に作成されて、クライアントの終了とともに削除される。サーバプロセスの終了時も削除される。
  - /tmp/denvl/pipes/<uuid>.client2server.pipe
  - /tmp/denvl/pipes/<uuid>.server2client.pipe

モジュール分け

- main.rs
- commands: サブコマンドに関するやつ
  - run
  - shutdown
  - \_\_server
- client: クライアントに関するやつ
- server: サーバに関するやつ
- communicate: サーバとクライアント間の通信に関するやつ
  - connection: named_pipe と protocol の合わせ技
  - named_pipe.rs: unix fifo のラップする
  - protocol
    - jsonrpc.rs // jsonrpc 2.0
    - cmdln_protocol.rs // JSONRPC v2.0 上のコマンドラインのプロトコル
    - lsp_protocol.rs
- core: 言語のそのものに関するやつ
  - parse
    - lex.rs
    - parse.rs
  - syntax_node
  - source
- util
