import * as Dialog from "@radix-ui/react-dialog";
import * as Slider from "@radix-ui/react-slider";
import * as Switch from "@radix-ui/react-switch";
import { X } from "lucide-react";

import type { AppSettings } from "../../contracts";

interface SettingsPanelProps {
  open: boolean;
  settings: AppSettings;
  onOpenChange: (open: boolean) => void;
  onChange: (settings: AppSettings) => void;
}

export function SettingsPanel({
  open,
  settings,
  onOpenChange,
  onChange,
}: SettingsPanelProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content className="settings-panel" aria-describedby={undefined}>
          <div className="settings-heading">
            <div>
              <p className="section-kicker">PREFERENCES</p>
              <Dialog.Title>流光设置</Dialog.Title>
            </div>
            <Dialog.Close asChild>
              <button type="button" className="icon-button" aria-label="关闭设置">
                <X size={18} aria-hidden="true" />
              </button>
            </Dialog.Close>
          </div>

          <div className="settings-group">
            <label htmlFor="theme-select">主题</label>
            <select
              id="theme-select"
              value={settings.theme}
              onChange={(event) =>
                onChange({
                  ...settings,
                  theme: event.target.value as AppSettings["theme"],
                })
              }
            >
              <option value="dark">深色</option>
              <option value="light">浅色</option>
              <option value="system">跟随系统</option>
            </select>
          </div>

          <div className="settings-group">
            <div className="settings-row">
              <span>便捷条宽度</span>
              <output>{settings.quickBarWidthCm.toFixed(1)} cm</output>
            </div>
            <Slider.Root
              className="slider"
              min={5}
              max={7}
              step={0.1}
              value={[settings.quickBarWidthCm]}
              onValueChange={([quickBarWidthCm]) =>
                onChange({ ...settings, quickBarWidthCm })
              }
              aria-label="便捷条宽度"
            >
              <Slider.Track className="slider-track">
                <Slider.Range className="slider-range" />
              </Slider.Track>
              <Slider.Thumb className="slider-thumb" />
            </Slider.Root>
          </div>

          <SettingSwitch
            label="启动时显示便捷条"
            checked={settings.showQuickBarOnLaunch}
            onChange={(showQuickBarOnLaunch) =>
              onChange({ ...settings, showQuickBarOnLaunch })
            }
          />
          <SettingSwitch
            label="开机启动"
            checked={settings.startWithWindows}
            onChange={(startWithWindows) =>
              onChange({ ...settings, startWithWindows })
            }
          />
          <SettingSwitch
            label="减少透明效果"
            checked={settings.reduceTransparency}
            onChange={(reduceTransparency) =>
              onChange({ ...settings, reduceTransparency })
            }
          />

          <div className="shortcut-list">
            <div>
              <span>临时解锁便捷条</span>
              <input
                aria-label="临时解锁快捷键"
                value={settings.unlockShortcut}
                onChange={(event) =>
                  onChange({
                    ...settings,
                    unlockShortcut: event.target.value,
                  })
                }
              />
            </div>
            <div>
              <span>显示/隐藏便捷条</span>
              <input
                aria-label="便捷条显示快捷键"
                value={settings.toggleQuickBarShortcut}
                onChange={(event) =>
                  onChange({
                    ...settings,
                    toggleQuickBarShortcut: event.target.value,
                  })
                }
              />
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

interface SettingSwitchProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function SettingSwitch({ label, checked, onChange }: SettingSwitchProps) {
  return (
    <div className="settings-group settings-switch-row">
      <label>{label}</label>
      <Switch.Root
        className="switch"
        checked={checked}
        onCheckedChange={onChange}
        aria-label={label}
      >
        <Switch.Thumb className="switch-thumb" />
      </Switch.Root>
    </div>
  );
}

