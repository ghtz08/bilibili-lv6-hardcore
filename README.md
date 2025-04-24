# B 站硬核会员自动做题

通过 ADB 和手机通信，使用多模态模型接口做题并自动点击选项回答，100 道题预计 5min 答完。

## 构建
```bash
cargo build --release
```

## 使用

1. `adb devices` 能看到设备并且能通过 `adb shell input` 模拟点击屏幕。
2. 多模态模型 API 接口服务。
3. 打开手机 B站，进入到做题页面之后运行程序。
