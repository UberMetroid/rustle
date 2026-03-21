import './App.css'

import { ClockIcon } from '@heroicons/react/outline'
import { format } from 'date-fns'
import { default as GraphemeSplitter } from 'grapheme-splitter'
import { useEffect, useState } from 'react'

import { AlertContainer } from './components/alerts/AlertContainer'
import { Grid } from './components/grid/Grid'
import { Keyboard } from './components/keyboard/Keyboard'
import { DatePickerModal } from './components/modals/DatePickerModal'
import { InfoModal } from './components/modals/InfoModal'
import { MigrateStatsModal } from './components/modals/MigrateStatsModal'
import { SettingsModal } from './components/modals/SettingsModal'
import { StatsModal } from './components/modals/StatsModal'
import { Navbar } from './components/navbar/Navbar'
import {
  DATE_LOCALE,
  DISCOURAGE_INAPP_BROWSERS,
  LONG_ALERT_TIME_MS,
  MAX_CHALLENGES,
  REVEAL_TIME_MS,
  WELCOME_INFO_MODAL_MS,
} from './constants/settings'
import {
  CORRECT_WORD_MESSAGE,
  DISCOURAGE_INAPP_BROWSER_TEXT,
  GAME_COPIED_MESSAGE,
  HARD_MODE_ALERT_MESSAGE,
  NOT_ENOUGH_LETTERS_MESSAGE,
  SHARE_FAILURE_TEXT,
  WIN_MESSAGES,
  WORD_NOT_FOUND_MESSAGE,
} from './constants/strings'
import { useAlert } from './context/AlertContext'
import { isInAppBrowser } from './lib/browser'
import {
  loadGameStateFromLocalStorage,
  saveGameStateToLocalStorage,
} from './lib/localStorage'
import { addStatsForCompletedGame, loadStats } from './lib/stats'
import {
  findFirstUnusedReveal,
  getGameDate,
  getIsLatestGame,
  getSolution,
  isWinningWord,
  isWordInWordList,
  setGameDate,
  unicodeLength,
} from './lib/words'
import init from './wordle-engine-pkg'

function App() {
  const isLatestGame = getIsLatestGame()
  const gameDate = getGameDate()

  const { showError: showErrorAlert, showSuccess: showSuccessAlert } =
    useAlert()
  const [currentGuess, setCurrentGuess] = useState('')
  const [isGameWon, setIsGameWon] = useState(false)
  const [isInfoModalOpen, setIsInfoModalOpen] = useState(false)
  const [isStatsModalOpen, setIsStatsModalOpen] = useState(false)
  const [isDatePickerModalOpen, setIsDatePickerModalOpen] = useState(false)
  const [isMigrateStatsModalOpen, setIsMigrateStatsModalOpen] = useState(false)
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false)
  const [currentRowClass, setCurrentRowClass] = useState('')
  const [isGameLost, setIsGameLost] = useState(false)
  const [isWasmReady, setIsWasmReady] = useState(false)
  const [solutionData, setSolutionData] = useState<any>(null)

  const [currentTheme, setCurrentTheme] = useState(
    localStorage.getItem('color-theme') || 'default'
  )
  const [isRevealing, setIsRevealing] = useState(false)
  const [guesses, setGuesses] = useState<string[]>([])
  const [stats, setStats] = useState(() => loadStats())

  const [isHardMode, setIsHardMode] = useState(
    localStorage.getItem('gameMode')
      ? localStorage.getItem('gameMode') === 'hard'
      : false
  )

  // Non-blocking Wasm init
  useEffect(() => {
    init()
      .then(() => {
        const data = getSolution(gameDate)
        if (data) {
          setSolutionData(data)
          setIsWasmReady(true)

          const loaded = loadGameStateFromLocalStorage(isLatestGame)
          if (loaded?.solution === data.solution) {
            setGuesses(loaded.guesses)
            if (loaded.guesses.includes(data.solution)) {
              setIsGameWon(true)
            } else if (loaded.guesses.length === MAX_CHALLENGES) {
              setIsGameLost(true)
            }
          }
        }
      })
      .catch(console.error)
  }, [gameDate, isLatestGame])

  useEffect(() => {
    if (isWasmReady && !loadGameStateFromLocalStorage(true)) {
      setTimeout(() => {
        setIsInfoModalOpen(true)
      }, WELCOME_INFO_MODAL_MS)
    }
  }, [isWasmReady])

  useEffect(() => {
    DISCOURAGE_INAPP_BROWSERS &&
      isInAppBrowser() &&
      showErrorAlert(DISCOURAGE_INAPP_BROWSER_TEXT, {
        persist: false,
        durationMs: 7000,
      })
  }, [showErrorAlert])

  useEffect(() => {
    const themes = [
      'theme-cyberpunk',
      'theme-nord',
      'theme-retro',
      'theme-solarized',
      'theme-default',
    ]
    themes.forEach((t) => document.documentElement.classList.remove(t))
    document.documentElement.classList.add(`theme-${currentTheme}`)
  }, [currentTheme])

  const handleHardMode = (isHard: boolean) => {
    if (guesses.length === 0 || localStorage.getItem('gameMode') === 'hard') {
      setIsHardMode(isHard)
      localStorage.setItem('gameMode', isHard ? 'hard' : 'normal')
    } else {
      showErrorAlert(HARD_MODE_ALERT_MESSAGE)
    }
  }

  const handleTheme = (theme: string) => {
    setCurrentTheme(theme)
    localStorage.setItem('color-theme', theme)
  }

  const clearCurrentRowClass = () => {
    setCurrentRowClass('')
  }

  useEffect(() => {
    if (solutionData) {
      saveGameStateToLocalStorage(getIsLatestGame(), {
        guesses,
        solution: solutionData.solution,
      })
    }
  }, [guesses, solutionData])

  useEffect(() => {
    if (isGameWon && solutionData) {
      const winMessage =
        WIN_MESSAGES[Math.floor(Math.random() * WIN_MESSAGES.length)]
      const delayMs = REVEAL_TIME_MS * solutionData.solution.length
      showSuccessAlert(winMessage, {
        delayMs,
        onClose: () => setIsStatsModalOpen(true),
      })
    }
    if (isGameLost && solutionData) {
      setTimeout(
        () => setIsStatsModalOpen(true),
        (solutionData.solution.length + 1) * REVEAL_TIME_MS
      )
    }
  }, [isGameWon, isGameLost, showSuccessAlert, solutionData])

  const onChar = (value: string) => {
    if (guesses.length < MAX_CHALLENGES && !isGameWon) {
      setCurrentGuess(
        `${currentGuess}${value}`.slice(0, solutionData?.solution.length || 5)
      )
    }
  }

  const onDelete = () => {
    setCurrentGuess(
      new GraphemeSplitter().splitGraphemes(currentGuess).slice(0, -1).join('')
    )
  }

  const onEnter = () => {
    if (!isWasmReady || isGameWon || isGameLost || !solutionData) return

    if (unicodeLength(currentGuess) !== solutionData.solution.length) {
      setCurrentRowClass('jiggle')
      return showErrorAlert(NOT_ENOUGH_LETTERS_MESSAGE, {
        onClose: clearCurrentRowClass,
      })
    }

    if (!isWordInWordList(currentGuess)) {
      setCurrentRowClass('jiggle')
      return showErrorAlert(WORD_NOT_FOUND_MESSAGE, {
        onClose: clearCurrentRowClass,
      })
    }

    if (isHardMode) {
      const firstMissingReveal = findFirstUnusedReveal(
        currentGuess,
        guesses,
        solutionData.solution
      )
      if (firstMissingReveal) {
        setCurrentRowClass('jiggle')
        return showErrorAlert(firstMissingReveal, {
          onClose: clearCurrentRowClass,
        })
      }
    }

    setIsRevealing(true)
    setTimeout(
      () => setIsRevealing(false),
      REVEAL_TIME_MS * solutionData.solution.length
    )

    const winningWord = isWinningWord(currentGuess, solutionData.solution)
    setGuesses([...guesses, currentGuess])
    setCurrentGuess('')

    if (winningWord) {
      if (isLatestGame)
        setStats(addStatsForCompletedGame(stats, guesses.length))
      return setIsGameWon(true)
    }

    if (guesses.length === MAX_CHALLENGES - 1) {
      if (isLatestGame)
        setStats(addStatsForCompletedGame(stats, guesses.length + 1))
      setIsGameLost(true)
      showErrorAlert(CORRECT_WORD_MESSAGE(solutionData.solution), {
        persist: true,
        delayMs: REVEAL_TIME_MS * solutionData.solution.length + 1,
      })
    }
  }

  return (
    <div className="flex h-full flex-col">
      <Navbar
        setIsInfoModalOpen={setIsInfoModalOpen}
        setIsStatsModalOpen={setIsStatsModalOpen}
        setIsDatePickerModalOpen={setIsDatePickerModalOpen}
        setIsSettingsModalOpen={setIsSettingsModalOpen}
      />

      {!isLatestGame && (
        <div className="flex items-center justify-center">
          <ClockIcon className="h-6 w-6 stroke-gray-600" />
          <p className="text-base text-gray-600">
            {format(gameDate, 'd MMMM yyyy', { locale: DATE_LOCALE })}
          </p>
        </div>
      )}

      <div className="mx-auto flex w-full max-w-[500px] grow flex-col justify-between px-1 py-2 sm:px-6">
        <div className="flex grow flex-col justify-center">
          <Grid
            solution={solutionData?.solution || '     '}
            guesses={guesses}
            currentGuess={currentGuess}
            isRevealing={isRevealing}
            currentRowClassName={currentRowClass}
          />
        </div>
        <div className="pb-2">
          <Keyboard
            onChar={onChar}
            onDelete={onDelete}
            onEnter={onEnter}
            solution={solutionData?.solution || '     '}
            guesses={guesses}
            isRevealing={isRevealing}
          />
        </div>
        <InfoModal
          isOpen={isInfoModalOpen}
          handleClose={() => setIsInfoModalOpen(false)}
        />
        <StatsModal
          isOpen={isStatsModalOpen}
          handleClose={() => setIsStatsModalOpen(false)}
          solution={solutionData?.solution || ''}
          solutionIndex={solutionData?.solutionIndex || 0}
          solutionGameDate={solutionData?.solutionGameDate || new Date()}
          tomorrow={solutionData?.tomorrow || 0}
          guesses={guesses}
          gameStats={stats}
          isLatestGame={isLatestGame}
          isGameLost={isGameLost}
          isGameWon={isGameWon}
          handleShareToClipboard={() => showSuccessAlert(GAME_COPIED_MESSAGE)}
          handleShareFailure={() =>
            showErrorAlert(SHARE_FAILURE_TEXT, {
              durationMs: LONG_ALERT_TIME_MS,
            })
          }
          handleMigrateStatsButton={() => {
            setIsStatsModalOpen(false)
            setIsMigrateStatsModalOpen(true)
          }}
          isHardMode={isHardMode}
          numberOfGuessesMade={guesses.length}
        />
        <DatePickerModal
          isOpen={isDatePickerModalOpen}
          initialDate={solutionData?.solutionGameDate || new Date()}
          handleSelectDate={(d) => {
            setIsDatePickerModalOpen(false)
            setGameDate(d)
          }}
          handleClose={() => setIsDatePickerModalOpen(false)}
        />
        <MigrateStatsModal
          isOpen={isMigrateStatsModalOpen}
          handleClose={() => setIsMigrateStatsModalOpen(false)}
        />
        <SettingsModal
          isOpen={isSettingsModalOpen}
          handleClose={() => setIsSettingsModalOpen(false)}
          isHardMode={isHardMode}
          handleHardMode={handleHardMode}
          currentTheme={currentTheme}
          handleTheme={handleTheme}
        />
        <AlertContainer />
      </div>
    </div>
  )
}

export default App
