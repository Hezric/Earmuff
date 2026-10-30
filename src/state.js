export const localeFor = language => language?.toLowerCase().startsWith('zh') ? 'zh' : 'en';
export function mockState() {
  return { cap: 42, drop: 10, current: 'headset', currentVolume: 38, pauseRemaining: 0, reduction: null, error: null, languageMode: 'system', pauseMinutes: 60,
    devices: [
      { id: 'headset', name: 'OPPO Enco Free4', protected: true, folded: false },
      { id: 'usb', name: 'Creative Stage SE', protected: false, folded: false },
      { id: 'virtual', name: 'CABLE Input (VB-Audio Virtual Cable)', protected: true, folded: true },
      { id: 'virtual16', name: 'CABLE In 16ch (VB-Audio Virtual Cable)', protected: true, folded: true },
      { id: 'stream', name: 'Steam Streaming Speakers', protected: true, folded: true },
      { id: 'display', name: 'PHL 245E1 (NVIDIA High Definition Audio)', protected: true, folded: true },
    ] };
}
export function mockUpdate(state, action) {
  if (action.type === 'setProtected') state.devices.find(d => d.id === action.id).protected = action.protected;
  if (action.type === 'setLevels') { state.cap = action.cap; state.drop = action.drop; }
  if (action.type === 'setPauseMinutes') state.pauseMinutes = action.minutes;
  if (action.type === 'setLanguage') state.languageMode = action.language;
  if (action.type === 'togglePause') { state.pauseRemaining = state.pauseRemaining ? 0 : state.pauseMinutes * 60; state.reduction = null; }
  return state;
}
export const text = {
  en: { current: 'Current', protected: 'Protected', unprotected: 'Not protected', paused: 'Paused', cap: 'Volume limit', drop: 'Lower a jump to 100% to', devices: 'Output devices', others: 'Other devices', pause: 'Pause for 1 hour', resume: 'Resume now', remaining: 'resumes in', minutes: 'min', none: 'No output device', noDevices: 'Connect an output device to get started.', reduced: 'lowered to', error: 'Protection needs attention', failed: 'Could not save changes. Try again.', protect: 'Protect', settings: 'Settings', language: 'Language', system: 'Follow system', app: 'Earmuff volume protection', github: 'Open Earmuff on GitHub', protection: 'Protection', pauseFor: 'Pause for', pauseError: 'Enter a whole number from 1 to 720.', limit: 'limit' },
  zh: { current: '当前', protected: '受保护', unprotected: '未保护', paused: '已暂停', cap: '最高音量', drop: '跳到 100% 时降到', devices: '输出设备', others: '其他设备', pause: '暂停 1 小时', resume: '立即恢复', remaining: '恢复还需', minutes: '分钟', none: '没有输出设备', noDevices: '连接输出设备即可开始。', reduced: '已降到', error: '保护需要处理', failed: '无法保存更改，请重试。', protect: '保护', settings: '设置', language: '语言', system: '跟随系统', app: 'Earmuff 音量保护', github: '在 GitHub 上打开 Earmuff', protection: '保护', pauseFor: '暂停时长', pauseError: '请输入 1 到 720 之间的整数。', limit: '上限' },
};
export function parsePauseMinutes(value) {
  const text = value.trim();
  const minutes = Number(text);
  return /^\d+$/.test(text) && minutes >= 1 && minutes <= 720 ? minutes : null;
}
export function pauseLabel(minutes, language) {
  return language === 'zh' ? minutes % 60 === 0 ? `暂停 ${minutes / 60} 小时` : `暂停 ${minutes} 分钟` : minutes % 60 === 0 ? `Pause ${minutes / 60} hr` : `Pause ${minutes} min`;
}