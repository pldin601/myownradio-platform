import cn from 'classnames'
import { useRef, useState } from 'react'
import { ProgressOverlay } from '@/components/ChannelTracksList/ProgressOverlay'
import AnimatedBars from '@/icons/AnimatedBars'
import { Duration } from '@/components/Duration/Duration'
import { ThreeDots } from '@/icons/ThreeDots'
import { MenuItemType, useContextMenu } from '@/modules/ContextMenu'
import { CurrentTrack, TrackItem } from './types'

interface Props {
  track: TrackItem
  currentTrack: CurrentTrack | null
  index: number
  onRemoveFromLibrary: () => void
  onRemoveFromChannel?: () => void
}

export const TrackListItem: React.FC<Props> = ({ track, currentTrack, index }) => {
  const isCurrentTrack = currentTrack?.index === index
  const portalRef = useRef<HTMLDivElement | null>(null)
  const contextMenu = useContextMenu()

  const [isHoverLocked, setHoverLocked] = useState(false)

  function showMenu(position: { x: number; y: number }) {
    contextMenu.show(
      {
        position,
        portalElement: portalRef.current ?? undefined,
        menuItems: [
          {
            type: MenuItemType.Item,
            label: 'Remove from channel',
            onClick() {},
          },
          {
            type: MenuItemType.Item,
            label: 'Remove from library',
            onClick() {},
          },
        ],
      },
      () => setHoverLocked(false),
    )
    setHoverLocked(true)
  }

  return (
    <li
      key={track.trackId}
      className={cn([
        'flex items-center border-gray-800 h-12 relative cursor-pointer',
        { 'bg-morblue-600 text-gray-300': isCurrentTrack },
        { 'hover:bg-morblue-100': !isCurrentTrack && !isHoverLocked },
        { 'bg-morblue-200': !isCurrentTrack && isHoverLocked },
        'group',
      ])}
      onContextMenu={(ev) => {
        ev.preventDefault()
        showMenu({
          x: ev.clientX,
          y: ev.clientY,
        })
      }}
    >
      <div ref={portalRef} />
      {isCurrentTrack && currentTrack && (
        <div className={cn('h-full w-full bg-morblue-300 absolute')}>
          <ProgressOverlay position={currentTrack.position} duration={currentTrack.duration} />
        </div>
      )}
      <div className="p-2 pl-4 w-12 flex-shrink-0 z-10 text-right">
        {isCurrentTrack ? <AnimatedBars size={12} /> : <>{index + 1}</>}
      </div>
      <div className="p-2 w-full z-10 min-w-0">
        <div className={'truncate'}>{track.title}</div>
        {track.artist && <div className={'text-xs truncate'}>{track.artist}</div>}
      </div>
      {track.album && <div className="p-2 w-full z-10 truncate hidden xl:block">{track.album}</div>}
      <div className="p-2 w-20 flex-shrink-0 text-right z-10">
        <Duration millis={track.duration} />
      </div>
      <div
        className={cn([
          'p-2 pr-4 w-10 flex-shrink-0 text-right z-10 cursor-pointer',
          'opacity-0 group-hover:opacity-100',
          { 'opacity-100': isHoverLocked },
        ])}
      >
        <span
          onClick={(ev) => {
            ev.preventDefault()
            showMenu({
              x: ev.clientX,
              y: ev.clientY,
            })
          }}
        >
          <ThreeDots size={14} />
        </span>
      </div>
    </li>
  )
}