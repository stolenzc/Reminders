# Reminders

一个能够通过自然语言创建 Apple 提醒事项的命令行工具。

## 安装

### 环境要求

- macOS（需连接 Apple 提醒事项，仅支持 macOS）

## 快速开始

```bash
reminders add "明天下午3点开会"
```

## 待办

列表命令：

- [ ] 列出所有提醒事项
- [ ] 列出指定列表的提醒事项
- [ ] 列出今日的提醒事项

添加命令：

- [ ] 指定列表添加提醒事项

完成命令：

- [ ] 完成指定提醒事项

删除命令：

- [ ] 删除指定提醒事项
- [ ] 删除所有提醒事项

## 配置

配置文件位于 `~/.config/reminders/config.json`

示例配置文件：

```json
{
  "ai": {
    "api_url": "https://api.openai.com/v1/chat/completions",
    "api_key": "your-api-key-here",
    "model": "gpt-4-turbo-preview"
  },
  "default_list": "提醒事项",
  "default_reminder_minutes": [15, 30]
}
```

## 灵感来源

- [TickTick](https://ticktick.com/) / [滴答清单](https://dida365.com/home)

## 致谢

- [Apple Reminders](https://www.icloud.com/reminders) 是一款非常实用的提醒工具
- [Rust Team And Community](https://rust-lang.org/) 提供了优秀的语言和丰富的工具生态
