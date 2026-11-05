以下の文章は生成AIによって生成されたものに加筆したものです。

# In-Network Aggregation Simulator for Heterogeneous Federated Learning

本リポジトリは、研究論文 **"In-Network Aggregation in Federated Learning for Heterogeneous Resource - IEEE INFOCOM 2026"** の評価・実験で使用したシミュレータコードです。

---

## ⚠️ ご確認・注意事項

- **コードの位置づけ**
  - 本コードは論文の検証・実験を目的として作成した研究用プロトタイプです。プロダクションでの利用を想定した設計や完全なリファクタリングは行っていません。
- **動作確認**
  - 手元の実験環境（Rust1.97, ubuntu:22.04, 24.04）での動作を確認していますが、すべての環境での実行や結果の完全な再現を保証するものではありません。
- **利用・転載について**
  - 本リポジトリに含まれるコード・構成の無断転載および二次利用はご遠慮ください。

---

## 🛠 簡単な概要

- **研究テーマ**: リソース異種性を考慮したFederated Learningにおけるインネットワーク集約（In-Network Aggregation）による通信データ量の削減
- - **使用言語**: Rust

-
