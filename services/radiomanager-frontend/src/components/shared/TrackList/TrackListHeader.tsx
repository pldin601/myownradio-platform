import { ThreeDots } from '@/components/shared/TrackList/icons/ThreeDots'
import React from 'react'

export const TrackListHeader: React.FC = () => (
  <li className="grid gap-4 grid-cols-playlist-item text-gray-500 p-4 sticky top-0 bg-morblue-950 z-10">
    <div className={'flex justify-end'}>#</div>
    <div className={'flex'}>Title</div>
    <div className={'flex justify-end'}>⏱</div>
    <div className={'flex'}>Artist</div>
    <div className={'flex'}>Album</div>
    <div className={'flex'}>
      <ThreeDots size={14} />
    </div>
  </li>
)
