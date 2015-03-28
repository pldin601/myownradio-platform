package gemini.myownradio.engine;import gemini.myownradio.LHttp.LHttpProtocol;import gemini.myownradio.engine.buffer.ConcurrentBuffer;import gemini.myownradio.engine.entity.Stream;import gemini.myownradio.ff.FFEncoderBuilder;import gemini.myownradio.tools.MORLogger;import gemini.myownradio.tools.io.IcyOutputStream;import java.io.IOException;import java.io.InputStream;import java.io.OutputStream;import java.sql.SQLException;/** * Created by Roman Gemini on 02.10.14. */public class ListenRadio {    private LHttpProtocol exchange;    private IcyOutputStream os;    private ConcurrentBuffer broadcast;    private boolean icy;    private Stream object;    private FFEncoderBuilder decoder;    private static MORLogger logger = new MORLogger(MORLogger.MessageKind.SERVER);    public ListenRadio(LHttpProtocol exchange, boolean icy, ConcurrentBuffer broadcast, FFEncoderBuilder decoder, Stream streamObject) {        this.broadcast = broadcast;        this.icy = icy;        this.object = streamObject;        this.decoder = decoder;        this.exchange = exchange;        logger.sprintf("Initializing listening session for client IP=%s", exchange.getClientIP());    }    public void listen() throws IOException, SQLException {        exchange.setContentType(decoder.getAudioFormat().getContent());        OutputStream sw;        if (icy) {            logger.sprintf("Client supports icy metadata");            exchange.setHeader("icy-metadata", "1");            exchange.setHeader("icy-name", object.getName() + " @ " + (broadcast.getStreamKey().getBitrate() / 1000) + "K");            exchange.setHeader("icy-metaint", Integer.toString(IcyOutputStream.DEFAULT_META_INTERVAL));            os = new IcyOutputStream(exchange.getOutputStream());            os.setTitle(object.getName());            sw = os;        } else {            sw = exchange.getOutputStream();        }        int len;        byte[] buff = new byte[4096];        boolean first = true;        boolean isMP3 = decoder.getAudioFormat().getCodec().equals("libmp3lame");        String prev = "";        FlowListener client = new FlowListener(exchange.getClientIP(),                exchange.getHeader("User-Agent"), decoder.getAudioFormatName(), object.getId());        logger.sprintf("Listening");        try (InputStream is = broadcast.getInputStream()) {            while ((len = is.read(buff)) != 0) {                if (icy && !prev.equals(broadcast.getTitle())) {                    prev = broadcast.getTitle();                    os.setTitle(prev);                }                if (isMP3 && first) {                    buff = trimContents(buff, len);                    first = false;                    sw.write(buff);                } else {                    sw.write(buff, 0, len);                }            }        } finally {            client.finish();        }    }    private byte[] getFLVHeader() {        return new byte[] { 'F', 'L', 'V', 0x01, 0x05, 0x09, 0x09 };    }    private byte[] trimContents(byte[] b, int len) {        byte[] header = new byte[] { (byte) 0xFF, (byte) 0xFB };        int end = len - 1;        for (int i = 0; i < end; i++) {            if (b[i] == header[0] && b[i + 1] == header[1]) {                byte[] temp = new byte[len - i];                logger.sprintf("Skipping %d bytes before mp3 header", i);                System.arraycopy(b, i, temp, 0, temp.length);                return temp;            }        }        byte[] temp = new byte[len];        System.arraycopy(b, 0, temp, 0, temp.length);        return temp;    }}