import { createClient } from "@redis/client"
import express from "express"
import makeDebug from "debug"
import { Config } from "./config.js"

const debug = makeDebug("main")

async function main() {
  const config = Config.fromEnv(process.env)

  // Initialize Redis client
  const redisClient = createClient({
    url: `redis://${process.env.REDIS_HOST}`,
  })
  await redisClient.connect()
  const app = express()

  app.post("/channel/:channelId/publish", async (req, res) => {
    const userId = req.headers["user-id"]
    const channelId = req.params["channelId"]

    if (!userId || Array.isArray(userId)) {
      return res.status(400).send("User-Id header is missing or invalid.")
    }

    try {
      const redisChannel = `${userId}_${channelId}`
      await redisClient.publish(redisChannel, req.body)

      res.status(200).send("OK")
    } catch (e) {
      debug("Error when publishing a message: %s", e)
      res.status(500).send("Internal server error.")
    }
  })

  app.get("/channel/:channelId/subscribe", async (req, res) => {
    const userId = req.headers["user-id"]
    const channelId = req.params["channelId"]

    if (!userId || Array.isArray(userId)) {
      return res.status(400).send("User-Id header is missing or invalid.")
    }

    const subscription = redisClient.duplicate()
    await subscription.connect()

    res.set({
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
    })

    const redisChannel = `${userId}_${channelId}`

    await subscription.subscribe(redisChannel, (message) => {
      res.write(`data: ${message}\n\n`)
    })

    req.on("close", async () => {
      await subscription.unsubscribe(redisChannel)
      await subscription.disconnect()
    })
  })

  app.listen(config.httpPort, () => {
    debug(`Server is running on port ${config.httpPort}`)
  })
}

main().catch((error) => {
  debug("Error when starting the server: %s", error)
  process.exit(1)
})
