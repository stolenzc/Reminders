# Reminders CLI

A cli tool which can parse input and create reminders in Apple Reminders.app.

## Installation

### Requirements

- macOS (connect to Apple Reminders, Only working on Macos)

## Quick Start

```bash
reminders add "明天下午3点开会"
```

## TODO

List command:

- [ ] List all Reminders
- [ ] List specify list Reminders
- [ ] List today Reminders

Add command:

- [ ] Specify a list to add reminder

Complete command:

- [ ] Complete specify reminder

Delete command:

- [ ] Delete specify reminder
- [ ] Delete all reminders

## Config

Config file location at `~/.config/reminders/config.json`

example config file

```json
{
  "ai": {
    "api_url": "https://api.openai.com/v1/chat/completions",
    "api_key": "your-api-key-here",
    "model": "gpt-4-turbo-preview"
  },
  "default_list": "reminders",
  "default_reminder_minutes": [15, 30],
}
```

## Inspired by

- [TickTick](https://ticktick.com/)

## Thanks

- [Apple Reminders](https://www.icloud.com/reminders) is a very aw
- [Rust Team And Community](https://rust-lang.org/) for providing a great language and rich tools.
