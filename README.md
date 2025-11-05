# BevyBot
`BevyBot`是一个专为`Bevy`引擎中文社区打造的自动化机器人，使用Rust语言编写。
通过`Github`提供的`API`，监控`Bevy`仓库的`Issue`和`PR`，并将更新内容发送到QQ频道中。
然后利用`DeepSeek AI`对重要`Issue`进行总结，帮助社区更好地了解项目动态。

## 开发说明

### 环境配置

复制 `.env.example` 文件，创建 `.env` 文件，并填写GITHUB_PERSON_TOKEN和GITHUB_PERSON_TOKEN等必要的环境变量。

### 自动编译

```bash
cargo watch -x run
```

### 首次生成实体

```bash
sea-orm-cli generate entity --with-serde both -o entity/src/entities
```
