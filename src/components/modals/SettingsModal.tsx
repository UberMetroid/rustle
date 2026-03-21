import {
  HARD_MODE_DESCRIPTION,
  HIGH_CONTRAST_MODE_DESCRIPTION,
} from '../../constants/strings'
import { BaseModal } from './BaseModal'
import { SettingsToggle } from './SettingsToggle'

type Props = {
  isOpen: boolean
  handleClose: () => void
  isHardMode: boolean
  handleHardMode: Function
  isDarkMode: boolean
  handleDarkMode: Function
  isHighContrastMode: boolean
  handleHighContrastMode: Function
  currentTheme: string
  handleTheme: Function
}

export const SettingsModal = ({
  isOpen,
  handleClose,
  isHardMode,
  handleHardMode,
  isDarkMode,
  handleDarkMode,
  isHighContrastMode,
  handleHighContrastMode,
  currentTheme,
  handleTheme,
}: Props) => {
  return (
    <BaseModal title="Settings" isOpen={isOpen} handleClose={handleClose}>
      <div className="mt-2 flex flex-col divide-y">
        <SettingsToggle
          settingName="Hard Mode"
          flag={isHardMode}
          handleFlag={handleHardMode}
          description={HARD_MODE_DESCRIPTION}
        />
        <SettingsToggle
          settingName="Dark Mode"
          flag={isDarkMode}
          handleFlag={handleDarkMode}
        />
        <SettingsToggle
          settingName="High Contrast Mode"
          flag={isHighContrastMode}
          handleFlag={handleHighContrastMode}
          description={HIGH_CONTRAST_MODE_DESCRIPTION}
        />
        <div className="flex flex-col py-3">
          <div className="flex justify-between">
            <p className="text-gray-500 dark:text-gray-300">Theme</p>
            <select
              value={currentTheme}
              onChange={(e) => handleTheme(e.target.value)}
              className="rounded border border-gray-300 bg-white p-1 text-sm text-gray-700 focus:border-indigo-500 focus:outline-none dark:bg-slate-800 dark:text-gray-200"
            >
              <option value="default">Default</option>
              <option value="cyberpunk">Cyberpunk</option>
              <option value="nord">Nord</option>
              <option value="retro">Retro</option>
              <option value="solarized">Solarized</option>
            </select>
          </div>
        </div>
      </div>
    </BaseModal>
  )
}
