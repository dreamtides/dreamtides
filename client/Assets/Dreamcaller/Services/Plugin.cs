#nullable enable

using System;
using System.Runtime.InteropServices;
using System.Text;
using Dreamcaller.Schema;
using Dreamcaller.Utils;
using Newtonsoft.Json;

static class Plugin
{
    const int BufferSize = 1_000_000;

    public static ConnectResponse Connect(ConnectRequest request)
    {
        var serialized = JsonConvert.SerializeObject(request, Converter.Settings);
        var encoded = Encoding.UTF8.GetBytes(serialized);

        byte[] response = new byte[BufferSize];
        int responseLength = Errors.CheckNonNegative(
            dreamcaller_connect(encoded, encoded.Length, response, BufferSize));
        var json = Encoding.UTF8.GetString(response, 0, responseLength);
        var deserialized = JsonConvert.DeserializeObject<ConnectResponse>(json, Converter.Settings);
        return Errors.CheckNotNull(deserialized, "Error deserializing connect response");
    }

    public static PerformActionResponse PerformAction(PerformActionRequest request)
    {
        var serialized = JsonConvert.SerializeObject(request, Converter.Settings);
        var encoded = Encoding.UTF8.GetBytes(serialized);

        byte[] response = new byte[BufferSize];
        int responseLength = Errors.CheckNonNegative(
            dreamcaller_perform_action(encoded, encoded.Length, response, BufferSize));
        var json = Encoding.UTF8.GetString(response, 0, responseLength);
        var deserialized = JsonConvert.DeserializeObject<PerformActionResponse>(json, Converter.Settings);
        return Errors.CheckNotNull(deserialized, "Error deserializing action response");
    }

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamcaller_connect(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamcaller_perform_action(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);
}
