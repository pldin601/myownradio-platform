import { useCallback, useState } from 'react'
import {
  toChannelTrackEntry,
  toChannelTrackEntry2,
  ChannelTrackEntry,
} from '@/components/entries/ChannelPage/ChannelTracksList'
import { deleteTracksById, removeTracksFromChannelById } from '@/api'
import { getChannelTracksPage } from '@/api/radiomanager'
import { useNowPlaying } from '@/modules/NowPlaying'
import { useHandleChannelLastUploadedTrack } from './useHandleChannelLastUploadedTrack'

import type { UserChannelTrack } from '@/api'

export const useChannelPageStore = (
  channelId: number,
  initialTracks: readonly UserChannelTrack[],
  initialTotalCount: number,
) => {
  const { refresh: refreshNowPlaying } = useNowPlaying()

  const [trackEntries, setTrackEntries] = useState<readonly (ChannelTrackEntry | null)[]>(() => {
    const entries = new Array<ChannelTrackEntry | null>(initialTotalCount).fill(null)
    entries.splice(0, initialTracks.length, ...initialTracks.map(toChannelTrackEntry))

    return entries
  })

  const addTrackEntry = useCallback((track: UserChannelTrack) => {
    setTrackEntries((entries) => [...entries, toChannelTrackEntry(track)])
  }, [])

  useHandleChannelLastUploadedTrack(channelId, addTrackEntry)

  const handleDeletingTracks = (trackIds: readonly number[]) => {
    const idsSet = new Set(trackIds)
    const updatedTrackEntries = trackEntries.filter((track) => track && !idsSet.has(track.trackId))

    setTrackEntries(updatedTrackEntries)

    deleteTracksById(trackIds)
      .then(() => {
        refreshNowPlaying()
      })
      .catch((error) => {
        // Restore tracks after unsuccessful delete
        setTrackEntries(trackEntries)
      })
  }

  const handleRemovingTracksFromChannel = (uniqueIds: readonly string[]) => {
    const idsSet = new Set(uniqueIds)
    const updatedTrackEntries = trackEntries.filter((track) => track && !idsSet.has(track.uniqueId))

    setTrackEntries(updatedTrackEntries)

    removeTracksFromChannelById(uniqueIds, channelId)
      .then(() => {
        refreshNowPlaying()
      })
      .catch((error) => {
        // Restore tracks after unsuccessful delete
        setTrackEntries(trackEntries)
      })
  }

  const loadMoreTracks = useCallback(
    async (startIndex: number, endIndex: number, signal: AbortSignal) => {
      const requestOpts = {
        offset: startIndex,
        limit: endIndex - startIndex,
        signal,
      }

      const { items, totalCount } = await getChannelTracksPage(channelId, requestOpts)

      setTrackEntries((prevEntries) => {
        let nextEntries = [...prevEntries]
        nextEntries.splice(startIndex, items.length, ...items.map(toChannelTrackEntry2))

        if (totalCount > nextEntries.length) {
          nextEntries.push(...new Array<null>(totalCount - nextEntries.length).fill(null))
        } else if (totalCount < nextEntries.length) {
          nextEntries.splice(totalCount)
        }

        return nextEntries
      })
    },
    [channelId],
  )

  return {
    trackEntries,
    loadMoreTracks,
    handleDeletingTracks,
    handleRemovingTracksFromChannel,
  }
}
