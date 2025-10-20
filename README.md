# 字根练习器

个人为了能够更高效地学习形码字根而开发的一款字根练习器。

## 编译

```bash
# 必要组件
cargo install dioxus-cli@0.7.0-rc.2

# 本地调试用，localhost:8080
dx serve

# 构建release版
dx build --release
dx bundle --web --release
# 构建好的bundle位于 target/dx/zigen-trainer/release/web/public
```

## 修改与添加新方案

如果要添加新的方案，需要对 [`schemes.json`](./assets/trainer/schemes.json) 作出修改。

```json5
{
    "id": "xin_fangan_id",
    "full_name": "新方案名称",
    "zigen_url": "./zigen/xin_fangan_mabiao.json",
    "zigen_font": "./xin_fangan_ziti.woff" // 如果不需要字根字体集，可以留空
}
```

码表的格式（以 [`src/scheme.rs`](./src/scheme.rs) 的 `LoadedScheme` 为准，以下供参考）：

```
JSON根节点 = [聚类]

聚类 = {
    "type": ("类" | "混"),  // ‘类‘ 指聚类，’混‘ 指混淆
    "description": string, // 聚类/混淆集的描述，用户练习时敲击空格会显示在屏幕下方
    "groups": [归并]
}

归并 = {
    "zigens": [string], // 位于同一聚类，外貌相似（或者根源相同），并且在归并后，编码一样的字根
    "code": string,     // 归并集的编码
    "description": string, // 归并集的描述，用户练习时敲击空格会显示在输入栏下方
    
    // 归并集的分类，练习器会根据字根的分类与用户设置，推迟一些字根的出现时间。
    // 分类有四种：常用通用字根（通）、常用简体字根（简）、常用繁体字根（繁）、不常用字根（罕）
    "classify": ('通' | '简' | '繁' | '罕'),
}
```

此外也可以参考 [`yuhao_star.json`](./assets/trainer/zigen/yuhao_star.json)。
