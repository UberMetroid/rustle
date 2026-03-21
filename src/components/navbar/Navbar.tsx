import {
  CalendarIcon,
  ChartBarIcon,
  CogIcon,
  InformationCircleIcon,
} from '@heroicons/react/outline'

import { ENABLE_ARCHIVED_GAMES } from '../../constants/settings'
import { GAME_TITLE } from '../../constants/strings'

type Props = {
  setIsInfoModalOpen: (value: boolean) => void
  setIsStatsModalOpen: (value: boolean) => void
  setIsDatePickerModalOpen: (value: boolean) => void
  setIsSettingsModalOpen: (value: boolean) => void
}

export const Navbar = ({
  setIsInfoModalOpen,
  setIsStatsModalOpen,
  setIsDatePickerModalOpen,
  setIsSettingsModalOpen,
}: Props) => {
  return (
    <div className="navbar">
      <div className="navbar-content px-5 short:h-auto">
        <div className="flex">
          <InformationCircleIcon
            className="h-6 w-6 cursor-pointer"
            onClick={() => setIsInfoModalOpen(true)}
          />
          {ENABLE_ARCHIVED_GAMES && (
            <CalendarIcon
              className="ml-3 h-6 w-6 cursor-pointer"
              onClick={() => setIsDatePickerModalOpen(true)}
            />
          )}
        </div>
        <p className="text-xl font-bold">{GAME_TITLE}</p>
        <div className="right-icons">
          <ChartBarIcon
            className="mr-3 h-6 w-6 cursor-pointer"
            onClick={() => setIsStatsModalOpen(true)}
          />
          <CogIcon
            className="h-6 w-6 cursor-pointer"
            onClick={() => setIsSettingsModalOpen(true)}
          />
        </div>
      </div>
      <hr className="border-gray-500 opacity-30"></hr>
    </div>
  )
}
