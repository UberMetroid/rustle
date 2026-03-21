import { HARD_MODE_DESCRIPTION } from '../../constants/strings'
import { BaseModal } from './BaseModal'
import { SettingsToggle } from './SettingsToggle'

type Props = {
  isOpen: boolean
  handleClose: () => void
  isHardMode: boolean
  handleHardMode: Function
  currentTheme: string
  handleTheme: (theme: string) => void
}

export const SettingsModal = ({
  isOpen,
  handleClose,
  isHardMode,
  handleHardMode,
  currentTheme,
  handleTheme,
}: Props) => {
  const themes = [
    { name: 'retro', color: '#00ff41', bg: '#000000' },
    { name: 'cyberpunk', color: '#ff007f', bg: '#0d0221' },
    { name: 'nord', color: '#88c0d0', bg: '#2e3440' },
    { name: 'default', color: '#10b981', bg: '#ffffff' },
    { name: 'solarized', color: '#b58900', bg: '#fdf6e3' },
  ]

  return (
    <BaseModal title="Settings" isOpen={isOpen} handleClose={handleClose}>
      <div className="mt-2 flex flex-col divide-y">
        <SettingsToggle
          settingName="Hard Mode"
          flag={isHardMode}
          handleFlag={handleHardMode}
          description={HARD_MODE_DESCRIPTION}
        />
        
        <div className="flex flex-col py-3">
          <div className="flex items-center justify-between">
            <p className="text-gray-500">Themes</p>
            <div className="relative flex h-8 w-48 items-center">
              {/* Rainbow Spectrum Bar */}
              <div className="absolute h-1.5 w-full rounded-full bg-gradient-to-r from-black via-purple-900 via-blue-900 via-green-400 to-yellow-100" />
              
              {/* Discrete Circles (Stops) */}
              <div className="z-10 flex w-full justify-between px-1">
                {themes.map((t) => (
                  <button
                    key={t.name}
                    onClick={() => handleTheme(t.name)}
                    className={`h-5 w-5 rounded-full border-2 transition-transform hover:scale-125 ${
                      currentTheme === t.name 
                        ? 'border-white ring-2 ring-indigo-500 scale-125' 
                        : 'border-gray-400'
                    }`}
                    style={{ backgroundColor: t.color }}
                    title={t.name.charAt(0).toUpperCase() + t.name.slice(1)}
                  />
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </BaseModal>
  )
}
