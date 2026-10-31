# Earmuff

给突如其来的高音量，留一道安静的防线。

[English](README.md)

Earmuff 守候 Windows 默认输出设备。新设备默认受保护：音量突然跳到满格时降到
**10%**，平时超过 **42%** 时压回上限。每台设备都能单独开关保护。

点开托盘浮窗，就能看到当前设备。设置里可以调整音量限制、选择 1–720 分钟的暂停时长，
以及切换简体中文或英文。浮窗随 Windows 的主题与强调色变化，关闭后释放网页视图。

## 安装

从[发布页](https://github.com/Hezric/Earmuff/releases/latest)下载 Windows x64 便携包，
对照 `SHA256SUMS` 校验后解压，并保留 WebView2。启动即开启音量保护。
登录启动交给已有启动管理器。

## 构建

需要 Windows 11、MSVC Rust 工具链、Visual Studio C++ Build Tools、Node.js LTS
和 Microsoft Edge WebView2，以及 [Vite+](https://viteplus.dev/guide/)。

```powershell
#requires -Version 7.0
git clone https://github.com/Hezric/Earmuff.git
cd Earmuff
vp install --frozen-lockfile
vp exec tauri build --ci --no-bundle -- --locked
```

便携程序位于 `src-tauri/target/release/earmuff.exe`，启动后立即开启保护。
开机启动请交给 Lineup 等启动管理工具。

### 交给智能体

把下面这段话交给能构建 Windows 软件的智能体：

```text
从 https://github.com/Hezric/Earmuff/releases/latest 安装 Earmuff。
对照 SHA256SUMS 校验 Windows x64 便携包，解压并确认 earmuff.exe 存在，先不启动。
保留已有设置和登录启动登记。说明启动即开启音量保护，启动前取得授权。
通过我已有的 Skill 管理方式安装 skills/earmuff。若没有，请先询问，建议使用
Vercel skills CLI：npx skills add Hezric/Earmuff -g（https://github.com/vercel-labs/skills，
要求 Node.js 22.20 及以上）。也要确认安装后 Skill 的位置和完整内容。
```

## 使用

左键点击托盘图标开关浮窗，右键菜单可暂停、恢复或退出。右下角的齿轮切换设备与设置。
暂停预设为 15、30、60、120 分钟，也可以输入自定义分钟数，按 Enter 或移开焦点保存。
再次暂停会使用已保存的时长；重新启动程序会恢复保护。

虚拟输出和显示器音频默认收在“其他设备”。一旦用作默认输出，就会永久移到主列表。
关闭某台设备的保护开关后，Earmuff 不会调整它的音量，满格时也一样。

设置保存在 `%APPDATA%\Earmuff\Earmuff.ini`。文件无法读取时，程序会使用安全默认值
保护设备并显示错误；浮窗中的更改通过原子替换保存。

耳机的实心耳罩表示保护开启，空心耳罩表示当前输出不受保护，两条竖线表示暂停；所有状态保持完整对比度。浮窗使用 Windows 原生阴影和圆角；隐藏图标会在通知区角落打开浮窗，点击外部或按 Esc 关闭。

## 预览与参与

运行 `vp run dev`，在 `http://127.0.0.1:1420` 预览模拟设备，完全不会操作真实音频。
加上 `?lang=zh` 或 `?lang=en` 可预览语言，`&theme=light`、`&theme=dark`
仅用于浏览器预览主题。

[参与贡献](CONTRIBUTING.md) · [功能与检查](docs/features/volume-guard.md)
· [MIT 许可](LICENSE) · [第三方声明](THIRD-PARTY-NOTICES.md)

Earmuff 限制的是 Windows 音量百分比，而非实测声压。
