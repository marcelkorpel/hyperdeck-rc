<script lang="ts">
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/tauri";
    import { open, save } from "@tauri-apps/api/dialog";
    import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
    import { onDestroy, onMount, tick } from "svelte";

    let ip = "";
    let port = "9993";
    let unlisten: Function;
    let bufferDiv: HTMLDivElement;
    let buffer = "";
    let manualCommand = "";
    let diskList: any[] = [];
    let timeline: any[] = [];
    let totalDuration: number = 0;
    let timelinePosition: number = 0;
    let currentClip: number | undefined;
    let isPlaying: boolean = false;

    async function connect() {
        await invoke("connect", { host: ip, port: port });
    }

    async function write(command: string) {
        //addTextToBuffer("<span>&gt;" + command + "</span>");
        await invoke("write", { command: command });
    }

    async function clearTimeline() {
        currentClip = undefined;
        timeline = [];
        totalDuration = 0;
        timelinePosition = 0;
        // some sane defaults
        await write("remote: enable: true");
        await write("notify: timeline position: true");
        // clear the timeline
        await write("clips clear");
        // get the list
        await write("disk list");
    }

    async function sendPlay() {
        isPlaying = true;
        await write("play");
    }

    async function sendStop() {
        isPlaying = false;
        await write("stop");
    }

    async function skipPrevious() {
        if (timelinePosition === timeline[currentClip!].position) {
            await write("goto: clip id: -1");
        } else {
            await write("goto: clip: start");
        }
    }

    async function skipNext() {
        await write("goto: clip id: +1");
    }

    function parseText(text: string) {
        const rows = text.split("\r\n");

        switch (text.substring(0, 3)) {
            case "103": // unsupported
                removeLastClipAsItIsUnsupported();
                addTextToBuffer(
                    "You cannot mix different formats, framerates, etc., in a timeline!",
                );
                break;
            case "107": // timeline empty
                parseText(text.substring(20));
                break;
            case "200": // ok
                parseText(text.substring(8));
                break;
            case "206": // disk list
                fillDiskList(rows);
                break;
            case "500": // general asynchronous communication
                addTextToBuffer(text);
                break;
            case "514": // timeline position
                updateTimelinePosition(rows);
                break;
        }
    }

    function fillDiskList(rows: Array<string>) {
        diskList = [];
        rows.shift();
        rows.shift();
        let ignoreRestOfInput = false;

        rows.forEach((row) => {
            if (row !== "" && !ignoreRestOfInput) {
                const line = row.split(" ");
                diskList.push({
                    name: line.slice(1, length - 3).join(" "),
                    type: line[line.length - 3],
                    resolution: line[line.length - 2],
                    length: line[line.length - 1],
                    frames: parseTimeToFrames(
                        line[line.length - 1],
                        line[line.length - 2],
                    ),
                });
            } else {
                ignoreRestOfInput = true;
            }
        });
    }

    function parseTimeToFrames(time: string, resolution: string): number {
        const framesPerSecond: number = parseInt(
            resolution.substring(resolution.indexOf("p") + 1, 10),
        );

        const hours: number = parseInt(time.substring(0, 2), 10) * 60 * 60;
        const minutes: number = parseInt(time.substring(3, 5), 10) * 60;
        const seconds: number = parseInt(time.substring(6, 8), 10);
        const frames: number = parseInt(time.substring(9), 10);

        return (hours + minutes + seconds) * framesPerSecond + frames;
    }

    function formatFrames(frames: number, resolution: string): string {
        const framesPerSecond: number = parseInt(
            resolution.substring(resolution.indexOf("p") + 1),
            10,
        );

        const hours = Math.floor(frames / 3600 / framesPerSecond);
        frames -= hours * 3600 * framesPerSecond;
        const minutes = Math.floor(frames / 60 / framesPerSecond);
        frames -= minutes * 60 * framesPerSecond;
        const seconds = Math.floor(frames / framesPerSecond);
        frames -= seconds * framesPerSecond;

        return [pad(hours), pad(minutes), pad(seconds), pad(frames)].join(":");
    }

    function spanAroundFrames(time: string): string {
        return (
            time.substring(0, 8) +
            '<span class="frames">' +
            time.substring(8) +
            "</span>"
        );
    }

    $: formatTotalDuration = (): string => {
        if (totalDuration === 0) {
            return "00:00:00:00";
        } else {
            return formatFrames(
                totalDuration,
                diskList[timeline[0].clip].resolution,
            );
        }
    };

    function pad(part: number): string {
        return part.toString().padStart(2, "0");
    }

    function updateTimelinePosition(rows: Array<string>) {
        // timeline positions start at 1, so decrement the input
        timelinePosition = parseInt(rows[1].substring(10), 10) - 1;
        determineCurrentClip();
    }

    function binarySearch() {
        let m: number = 0;
        let n: number = timeline.length - 1;
        while (m <= n) {
            let k: number = (n + m) >> 1;
            let cmp: number = timelinePosition - timeline[k].position;
            if (cmp > 0) {
                m = k + 1;
            } else if (cmp < 0) {
                n = k - 1;
            } else {
                return k;
            }
        }
        return n;
    }

    function determineCurrentClip() {
        if (timeline.length === 0) {
            currentClip = undefined;
        } else {
            currentClip = binarySearch();
        }
    }

    async function addClip(index: number) {
        await write("clips add: name: " + diskList[index].name);

        timeline.push({
            clip: index,
            position: totalDuration,
        });
        timeline = timeline; // trigger reactivity
        totalDuration += diskList[timeline[timeline.length - 1].clip].frames;

        await tick();
    }

    async function openTimelineFile() {
        const selected = await open({
            filters: [
                {
                    name: "JSON file",
                    extensions: ["json"],
                },
            ],
        });

        if (selected !== null && !Array.isArray(selected)) {
            await clearTimeline();
            const contents = await readTextFile(selected);
            const fileList: string[] = JSON.parse(contents);

            fileList.forEach(async (name) => {
                for (let i = 0; i < diskList.length; i++) {
                    if (diskList[i].name === name) {
                        await addClip(i);
                        return;
                    }
                }

                // not found
                addTextToBuffer("Clip " + name + " cannot be found!");
            });
        }
    }

    async function saveTimelineFile() {
        const filePath = await save({
            filters: [
                {
                    name: "JSON file",
                    extensions: ["json"],
                },
            ],
        });

        if (filePath !== null) {
            let fileList: string[] = [];
            timeline.forEach((element) => {
                fileList.push(diskList[element.clip].name);
            });
            await writeTextFile(filePath, JSON.stringify(fileList));

            addTextToBuffer("Timeline has been saved to " + filePath);
        }
    }

    function removeLastClipAsItIsUnsupported() {
        totalDuration -= diskList[timeline[timeline.length - 1].clip].frames;
        timeline.pop();
        timeline = timeline; // trigger reactivity
    }

    async function addTextToBuffer(text: string) {
        buffer += text.replaceAll("\r\n", "<br>") + "<br>";
        await tick();
        bufferDiv.scrollTop = bufferDiv.scrollHeight;
    }

    onMount(async () => {
        unlisten = await listen("message", async (event: any) => {
            parseText(event.payload);
            //await addTextToBuffer(event.payload);
        });
    });

    onDestroy(() => unlisten());
</script>

<div class="container">
    <h1>HyperDeck player remote control</h1>

    <div class="main-panel">
        <form class="row" on:submit|preventDefault={connect}>
            <input
                placeholder="Enter IP address"
                bind:value={ip}
                disabled={isPlaying}
            />
            <input placeholder="Port" bind:value={port} disabled={isPlaying} />
            <button type="submit" disabled={isPlaying}>Connect</button>
        </form>

        <form class="row" on:submit|preventDefault={() => write(manualCommand)}>
            <input
                placeholder="Enter command"
                bind:value={manualCommand}
                disabled={isPlaying}
            />
            <button type="submit" disabled={isPlaying}>Send</button>
        </form>

        <div class="row">
            <button on:click={clearTimeline} disabled={isPlaying}
                >Clear timeline / reset</button
            >
            <button on:click={openTimelineFile} disabled={isPlaying}
                >Open</button
            >
            <button on:click={saveTimelineFile} disabled={isPlaying}
                >Save</button
            >
        </div>

        <div class="columns">
            <div class="disk-list column">
                <div class="scrollable">
                    {#each diskList as line, index}
                        <button
                            on:click={() => addClip(index)}
                            disabled={isPlaying}
                        >
                            <div>
                                <span class="name">{line.name}</span>
                                <span class="length"
                                    >{@html spanAroundFrames(line.length)}</span
                                >
                            </div>
                            <div>
                                <span class="metadata">
                                    {line.type}
                                    {line.resolution}
                                </span>
                            </div>
                        </button>
                    {/each}
                </div>
            </div>

            <div class="timeline column">
                <div class="scrollable">
                    {#each timeline as clip, k}
                        <div>
                            {#if k === currentClip}
                                <div class="diamond">stat_0</div>
                            {:else}
                                <div class="diamond invisible">stat_0</div>
                            {/if}
                            <div class="timeline-item">
                                <span class="position"
                                    >{@html spanAroundFrames(
                                        formatFrames(
                                            clip.position,
                                            diskList[clip.clip].resolution,
                                        ),
                                    )}</span
                                >
                                <span class="name"
                                    >{diskList[clip.clip].name}</span
                                >
                                <span class="length"
                                    >{@html spanAroundFrames(
                                        diskList[clip.clip].length,
                                    )}</span
                                >
                            </div>
                        </div>
                    {/each}
                    <div>
                        <div class="diamond invisible">stat_0</div>
                        <div class="timeline-item">
                            <span class="position end"
                                >{@html spanAroundFrames(
                                    formatTotalDuration(),
                                )}</span
                            >
                            <span class="name end">end of timeline</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div class="status">
        {#if totalDuration === 0}
            <div>Full timeline:</div>
            <div class="full-timeline">
                {@html spanAroundFrames("00:00:00:00")}
                /
                {@html spanAroundFrames("00:00:00:00")}
                <progress />
                {@html spanAroundFrames("00:00:00:00")}
            </div>
        {:else}
            <div>Full timeline:</div>
            <div class="full-timeline">
                {@html spanAroundFrames(
                    formatFrames(
                        timelinePosition,
                        diskList[timeline[0].clip].resolution,
                    ),
                )} /
                {@html spanAroundFrames(formatTotalDuration())}
                <progress value={timelinePosition} max={totalDuration} />
                {@html spanAroundFrames(
                    formatFrames(
                        totalDuration - timelinePosition,
                        diskList[timeline[0].clip].resolution,
                    ),
                )}
            </div>
        {/if}
        {#if currentClip === undefined}
            <div>Current clip: -</div>
            <div class="single-clip">
                {@html spanAroundFrames("00:00:00:00")}
                /
                {@html spanAroundFrames("00:00:00:00")}
                <progress />
                {@html spanAroundFrames("00:00:00:00")}
            </div>
        {:else}
            <div>Current clip: {diskList[timeline[currentClip].clip].name}</div>
            <div class="single-clip">
                {@html spanAroundFrames(
                    formatFrames(
                        timelinePosition - timeline[currentClip].position,
                        diskList[timeline[0].clip].resolution,
                    ),
                )} /
                {@html spanAroundFrames(
                    diskList[timeline[currentClip].clip].length,
                )}
                <progress
                    value={timelinePosition - timeline[currentClip].position}
                    max={diskList[timeline[currentClip].clip].frames}
                />
                {@html spanAroundFrames(
                    formatFrames(
                        diskList[timeline[currentClip].clip].frames -
                            (timelinePosition - timeline[currentClip].position),
                        diskList[timeline[0].clip].resolution,
                    ),
                )}
            </div>
        {/if}
    </div>

    <div class="row controls">
        <button on:click={skipPrevious} disabled={isPlaying}
            >skip_previous</button
        >
        <button on:click={sendPlay} disabled={isPlaying}>play_arrow</button>
        <button on:click={sendStop} disabled={!isPlaying}>pause</button>
        <button on:click={skipNext} disabled={isPlaying}>skip_next</button>
    </div>

    <div class="buffer" bind:this={bufferDiv}>{@html buffer}</div>
</div>

<style>
    @font-face {
        font-family: "Material Symbols Sharp";
        font-style: normal;
        src: url("/MaterialSymbolsSharp_Filled_36pt-Regular.ttf")
            format("truetype");
    }

    :root {
        font-family: -apple-system, sans-serif;
        font-variant-numeric: tabular-nums;
        font-size: 16px;
        line-height: 24px;
        color: #0f0f0f;
        background-color: #f6f6f6;
        margin: 0;
        padding: 0;
    }

    input,
    button {
        font-variant-numeric: tabular-nums;
    }

    .container {
        height: 100%;
        display: flex;
        flex-direction: column;
    }

    .main-panel {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    .columns {
        flex: 1;
        display: flex;
        height: 100%;
    }

    .column {
        position: relative;
        flex: 1;
    }

    .scrollable {
        position: absolute;
        width: 100%;
        height: 100%;
        overflow: auto;
    }

    :global(.frames) {
        color: #999;
    }

    .disk-list button {
        display: block;
        width: 100%;
        text-align: left;
        border: none;
        background: none;
        padding: 2px 12px;
        font-size: 16px;
    }

    .disk-list button:disabled {
        color: #999;
    }

    .metadata {
        font-size: 12px;
        color: #999;
    }

    .diamond {
        display: inline-block;
        width: 2em;
        font-family: "Material Symbols Sharp";
    }

    .diamond.invisible {
        visibility: hidden;
    }

    .timeline-item {
        display: inline-block;
    }

    .end {
        font-style: italic;
    }

    .status {
        margin: 10px 0;
        text-align: center;
        font-size: 16px;
    }

    .full-timeline {
        margin-bottom: 10px;
    }

    progress {
        appearance: none;
        width: calc(100% - 30rem);
        height: 16px;
        border: 0 none;
        background: lightgray;
        border-radius: 8px;
    }

    progress::-webkit-progress-bar {
        background: transparent;
    }

    progress::-webkit-progress-value {
        border-radius: 8px;
        background: orange;
    }

    .status .single-clip progress {
        position: relative;
        top: -0.2rem;
        height: 10px;
        border-radius: 5px;
    }

    .status .single-clip progress::-webkit-progress-value {
        border-radius: 5px;
    }

    .controls {
        text-align: center;
        margin: 10px 0;
    }

    .controls button {
        font-family: "Material Symbols Sharp";
        font-size: 36px;
    }

    .buffer {
        height: 150px;
        overflow: auto;
        background: white;
        font-size: 12px;
        line-height: 14px;
    }

    .buffer :global(span) {
        color: gray;
    }

    h1 {
        text-align: center;
    }
</style>
