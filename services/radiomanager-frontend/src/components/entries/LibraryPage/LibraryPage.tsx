'use client'

import { User, UserChannel, UserTrack } from '@/api/api.types'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { getLibraryTracks, deleteTracksById, MAX_TRACKS_PER_REQUEST } from '@/api/api.client'
import { LibraryLayout } from '@/components/layouts/LibraryLayout'
import { Header } from '@/components/Header'
import { Sidebar } from '@/components/Sidebar'
import {
  LibraryTracksList,
  toLibraryTrackEntry,
} from '@/components/LibraryTracksList/LibraryTracksList'
import { useMediaUploader, MediaUploaderComponent } from '@/modules/MediaUploader'

interface Props {
  user: User
  userTracks: readonly UserTrack[]
  userChannels: readonly UserChannel[]
}

export const LibraryPage: React.FC<Props> = ({ user, userTracks, userChannels }) => {
  const initialTrackEntries = useMemo(() => userTracks.map(toLibraryTrackEntry), [userTracks])
  const [trackEntries, setTrackEntries] = useState(initialTrackEntries)

  const addTrackEntry = useCallback((track: UserTrack) => {
    setTrackEntries((entries) => [toLibraryTrackEntry(track), ...entries])
  }, [])

  const removeTrackEntry = useCallback((indexToRemove: number) => {
    setTrackEntries((entries) => entries.filter((_, index) => index !== indexToRemove))
  }, [])

  const { lastUploadedTrack } = useMediaUploader()
  useEffect(() => {
    if (!lastUploadedTrack) {
      return
    }

    addTrackEntry(lastUploadedTrack.track)
  }, [lastUploadedTrack, addTrackEntry])

  const initialCanInfinitelyScroll = initialTrackEntries.length === MAX_TRACKS_PER_REQUEST
  const [canInfinitelyScroll, setCanInfinitelyScroll] = useState(initialCanInfinitelyScroll)
  const handleInfiniteScroll = () => {
    getLibraryTracks(trackEntries.length).then((tracks) => {
      const newEntries = tracks.map(toLibraryTrackEntry)
      setTrackEntries((entries) => [...entries, ...newEntries])

      if (MAX_TRACKS_PER_REQUEST > newEntries.length) {
        setCanInfinitelyScroll(newEntries.length === MAX_TRACKS_PER_REQUEST)
      }
    })
  }

  const handleDeletingTracks = (trackIds: readonly number[]) => {
    const idsSet = new Set(trackIds)
    const updatedTrackEntries = trackEntries.filter(({ trackId }) => !idsSet.has(trackId))

    setTrackEntries(updatedTrackEntries)

    deleteTracksById(trackIds).catch((error) => {
      // Restore tracks after unsuccessful delete
      setTrackEntries(trackEntries)
    })
  }

  return (
    <>
      <LibraryLayout
        header={<Header user={user} />}
        sidebar={<Sidebar channels={userChannels} activeItem={['library']} />}
        content={
          <LibraryTracksList
            tracks={trackEntries}
            canInfinitelyScroll={canInfinitelyScroll}
            onInfiniteScroll={handleInfiniteScroll}
            onDeleteTracks={handleDeletingTracks}
          />
        }
        rightSidebar={null}
      />
      <MediaUploaderComponent />
    </>
  )
}

export const LibraryPageWithProviders: React.FC<Props> = (props) => {
  return <LibraryPage {...props} />
}