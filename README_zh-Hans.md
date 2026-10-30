# Earmuff

给突如其来的高音量，留一道安静的防线。

[English](README.md)

Earmuff 守候 Windows 默认输出设备。新设备默认受保护：音量突然跳到满格时降到
**10%**，平时超过 **42%** 时压回上限。每台设备都能单独开关保护。

点开托盘浮窗，就能看到当前设备。设置里可以调整音量限制、选择 1–720 分钟的暂停时长，
以及切换简体中文或英文。浮窗随 Windows 的主题与强调色变化，关闭后释放网页视图。

## 构建

需要 Windows 11、MSVC Rust 工具链、Visual Studio C++ Build Tools、Node.js LTS
和 Microsoft Edge WebView2。目前尚未发布安装包。

```powershell
git clone https://github.com/Hezric/Earmuff.git
cd Earmuff
npm ci
npm run tauri -- build --ci --no-bundle -- --locked
```

便携程序位于 `src-tauri/target/release/earmuff.exe`，启动后立即开启保护。
开机启动请交给 Lineup 等启动管理工具。

### 交给智能体

把下面这段话交给能构建 Windows 软件的智能体：

```text
请从 https://github.com/Hezric/Earmuff 获取 Earmuff，在本地构建。
先阅读 AGENTS.md，检查 Windows 11、MSVC Rust/C++ 工具链、Node.js LTS
和 WebView2。运行 npm ci，然后运行
npm run tauri -- build --ci --no-bundle -- --locked。
确认生成 src-tauri/target/release/earmuff.exe，不要启动程序。
说明启动后会立即开启音量保护；启动程序或设置开机启动前先征得我的同意。
```

## 使用

左键点击托盘图标打开浮窗，右键菜单可暂停、恢复或退出。右下角的齿轮切换设备与设置。
暂停预设为 15、30、60、120 分钟，也可以输入自定义分钟数，按 Enter 或移开焦点保存。
再次暂停会使用已保存的时长；重新启动程序会恢复保护。

虚拟输出和显示器音频默认收在“其他设备”。一旦用作默认输出，就会永久移到主列表。
关闭某台设备的保护开关后，Earmuff 不会调整它的音量，满格时也一样。

设置保存在 `%APPDATA%\Earmuff\Earmuff.ini`。文件无法读取时，程序会使用安全默认值
保护设备并显示错误；浮窗中的更改通过原子替换保存。

## 预览与参与

运行 `npm run dev`，在 `http://127.0.0.1:1420` 预览模拟设备，完全不会操作真实音频。
加上 `?lang=zh` 或 `?lang=en` 可预览语言，`&theme=light`、`&theme=dark`
仅用于浏览器预览主题。

[参与贡献](CONTRIBUTING.md) · [功能与检查](docs/features/volume-guard.md)
· [MIT 许可](LICENSE) · [第三方声明](THIRD-PARTY-NOTICES.md)

Earmuff 限制的是 Windows 音量百分比，而非实测声压。
