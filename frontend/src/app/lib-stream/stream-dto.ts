import { StringDate, StringDateTime } from "../common/string-date-time";
import { StringDateTimeUtil } from "../utils/string-date-time.util";


export enum StreamState {
    waiting = "waiting",
    preparing = "preparing",
    started = "started",
    paused = "paused",
    stopped = "stopped"
}
export class StreamStateUtil {
    public static create(value: string): StreamState | null {
        let result: StreamState | null = null;
        switch (value) {
            case StreamState.waiting: result = StreamState.waiting; break;
            case StreamState.preparing: result = StreamState.preparing; break;
            case StreamState.started: result = StreamState.started; break;
            case StreamState.stopped: result = StreamState.stopped; break;
            case StreamState.paused: result = StreamState.paused; break;
        }
        return result;
    }
    public static isActive(streamState: StreamState): boolean {
        return [StreamState.preparing, StreamState.started, StreamState.paused].includes(streamState);
    }
}

export type StreamSateType = "waiting" | "preparing" | "started" | "paused" | "stopped";

export interface StreamDto {
    id: number;
    // Owner id
    userId: number;
    // Custom title (min: 2, max: 255)
    title: string;
    // Custom description (default: "") (min: 2, max: 2048)
    descript: string;
    // Link to stream logo, optional (min: 2, max: 255)
    logo: string | null;
    // The stream start time. Required on create
    starttime: Date | null;
    // Stream live status, false means inactive
    live: boolean;
    // Stream live state - waiting, preparing, start, paused, stop (waiting by default)
    state: StreamState; // ["waiting", "preparing", "started", "paused", "stopped"]
    // The time the stream began.
    started: Date | null;
    // The time the stream began pausing.
    paused: Date | null;
    // The time the stream stopped.
    stopped: Date | null;
    source: string;
    tags: string[];
    createdAt: Date | null;
    updatedAt: Date | null;
}

export class StreamDtoUtil {
    public static create(streamDto?: Partial<StreamDto>): StreamDto {
        return {
            id: (streamDto?.id || -1),
            userId: (streamDto?.userId || -1),
            title: (streamDto?.title || ""),
            descript: (streamDto?.descript || ""),
            logo: (streamDto?.logo || null),
            starttime: this.getDate(streamDto?.starttime || null),
            live: (streamDto?.live || false),
            started: this.getDate(streamDto?.started || null), // StringDateTime;
            paused: this.getDate(streamDto?.paused || null), // StringDateTime;
            stopped: this.getDate(streamDto?.stopped || null), // StringDateTime;
            state: (streamDto?.state || StreamState.waiting),
            tags: (streamDto?.tags || []),
            source: (streamDto?.source || "obs"),
            createdAt: this.getDate(streamDto?.createdAt || null), // StringDateTime;
            updatedAt: this.getDate(streamDto?.updatedAt || null), // StringDateTime;
        };
    }
    public static getDate(value: StringDateTime | Date | null): Date | null {
        const isString = value != null && typeof value == "string";
        return isString ? StringDateTimeUtil.toDate(value) : value;
    }
    public static isFuture(startTime: StringDateTime | null): boolean | null {
        let date: Date | null = StringDateTimeUtil.toDate(startTime);
        const now = new Date();
        //   return (!!startTime ? moment().isBefore(moment(startTime, MOMENT_ISO8601), "day") : null);
        return date != null ? (now < date) : null;
    }
    public static createList(streamList: Partial<StreamDto>[]): StreamDto[] {
        const result: StreamDto[] = [];
        for (let idx = 0; idx < streamList.length; idx++) {
            result.push(this.create(streamList[idx]));
        }
        return result;
    }
}

export interface PageStreamAndTagsDto {
    list: StreamDto[];
    limit: number;
    count: number;
    page: number;
    pages: number;
}

export interface UpdateStreamFileDto {
    id?: number | undefined;
    title?: string | undefined;
    descript?: string | undefined;
    starttime?: StringDateTime | null | undefined;
    source?: string | undefined;
    tags?: string[] | undefined;
    logoFile?: File | null | undefined;
}

// ** getStreams(), getStreamsByDate()  **

export interface SearchStreamAndTagsDto {
    userId?: number | undefined;
    live?: boolean | undefined;
    filter?: "future" | "past" | "period"
    starttime?: StringDateTime | null | undefined; // DateTime<Utc>,
    finishtime?: StringDateTime | null | undefined; // DateTime<Utc>,
    sortIdDesc?: boolean | undefined;
    tagName?: string | undefined;
    page?: number; // default 1;
    limit?: number; // default 10; Min(1) Max(100)
}

// ** getCalendarStreamsByDate()  **

export interface SearchStreamsDateDto {
    userId: number;
    start: StringDateTime;
    finish: StringDateTime;
}

export interface StreamsPeriodDto {
    date: StringDate;
    count: number;
}

// ** getStreamsPopularTags **

export type StreamTagSortColumnType = "id" | "name" | "countLinks";

export interface SearchStreamTagDto {
    sortColumn?: StreamTagSortColumnType;
    sortIdDesc?: boolean;
    page?: number;
    limit?: number;
}

export interface StreamTagDto {
    id: number;
    name: string;
    countLinks: number;
}

// ** **