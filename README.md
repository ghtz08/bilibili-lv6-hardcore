# B 站硬核会员自动做题

通过 ADB 和手机通信，使用多模态模型接口做题并自动点击选项回答，100 道题预计 6min 答完。

## 构建

```bash
cargo build --release
```

或者直接在 release 页面下载预构建的版本，目前仅在 linux 上测试过。

## 使用

### 准备

* `adb devices` 能看到设备并且能通过 `adb shell input` 模拟点击屏幕。
* 一个多模态模型 API 接口服务。

### 运行

* 打开手机 B站，选好题目类型进入到做题页面之后运行程序。
* 必须参数
  * --api-url: 指定多模态模型的 API。
  * --api-model: API 使用的模型。
  * --api-key: 用于验证的 API KEY。
* 可选参数
  * --api-cost-input: 配置 API 输入 token 的成本，用于计算最终成本。
  * --api-cost-output: 配置 API 输出 token 的成本，用于计算最终成本。
