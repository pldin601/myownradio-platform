import { useMediaUploader } from '@/modules/MediaUploader'
import useFileSelect from '@/hooks/useFileSelect'

interface Props {
  readonly targetChannelId?: number
}

const ACCEPT_CONTENT_TYPES = 'audio/*'

export const MediaUploader: React.FC<Props> = ({ targetChannelId }) => {
  const { upload, lastUploadedTrack, uploadQueue, uploadErrors } = useMediaUploader()
  const select = useFileSelect(ACCEPT_CONTENT_TYPES, (files) => files.forEach((f) => upload(f)))

  return null
  // return <>
  //   <button onClick={select}>Upload</button>
  //   <h1>Queue:</h1>
  //   <ul>
  //     {uploadQueue.map((item, i) => (
  //       <li key={i}>{item.file.name}</li>
  //     ))}
  //   </ul>
  //   <h1>Errors:</h1>
  //   <ul>
  //     {uploadErrors.map((item, i) => (
  //       <li key={i}>{item.error.message}</li>
  //     ))}
  //   </ul>
  // </>
}
