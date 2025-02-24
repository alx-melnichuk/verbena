import { StringDate, StringDateTime } from '../common/string-date-time';
import { StringDateTimeUtil } from '../utils/string-date-time.util';


export enum StreamState {
    waiting = 'waiting',
    preparing = 'preparing',
    started = 'started',
    paused = 'paused',
    stopped = 'stopped'
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

export type StreamSateType = 'waiting' | 'preparing' | 'started' | 'paused' | 'stopped';

export interface StreamDto {
    id: number;
    userId: number;
    title: string;
    descript: string; // description
    logo: string | null;
    starttime: StringDateTime | null;
    live: boolean;
    state: StreamState; // ['waiting', 'preparing', 'started', 'paused', 'stopped']
    started: StringDateTime | null; // Date | null;
    stopped: StringDateTime | null; // Date | null;
    // status: bool,
    source: string;
    tags: string[];
    isMyStream?: boolean;
    // credentials: Credentials | null;
    // publicTarget: string | null;
    createdAt: StringDateTime;
    updatedAt: StringDateTime;
}

export class StreamDtoUtil {
    public static create(streamDto?: Partial<StreamDto>): StreamDto {
        return {
            id: (streamDto?.id || -1),
            userId: (streamDto?.userId || -1),
            title: (streamDto?.title || ''),
            descript: (streamDto?.descript || ''),
            logo: (streamDto?.logo || null),
            starttime: (streamDto?.starttime || null), // Date;
            live: (streamDto?.live || false),
            started: (streamDto?.started || null),
            stopped: (streamDto?.stopped || null),
            state: (streamDto?.state || StreamState.waiting),
            tags: (streamDto?.tags || []),
            source: (streamDto?.source || 'obs'),
            isMyStream: (streamDto?.isMyStream),
            // credentials: (streamDto?.credentials || null),
            // publicTarget: (streamDto?.publicTarget || null)
            createdAt: (streamDto?.createdAt || ''),
            updatedAt: (streamDto?.updatedAt || ''),
        };
    }
    public static isFuture(startTime: StringDateTime | null): boolean | null {
        let date: Date | null = StringDateTimeUtil.toDate(startTime);
        const now = new Date();
        //   return (!!startTime ? moment().isBefore(moment(startTime, MOMENT_ISO8601), 'day') : null);
        return date != null ? (now < date) : null;
    }
}

export interface StreamListDto {
    list: StreamDto[];
    limit: number;
    count: number;
    page: number;
    pages: number;
}

export interface StreamEventDto {
    id: number;
    userId: number;
    title: string;
    logo: string | null;
    starttime: StringDateTime | null;
}

export interface StreamEventPageDto {
    list: StreamEventDto[];
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

// ** getStreams()  **

export interface SearchStreamDto {
    userId?: number;
    live?: boolean;
    // Future streams with a "starttime" greater than or equal to the specified one.
    futureStarttime?: StringDateTime | null | undefined; // DateTime<Utc>,
    // Past streams with a "starttime" greater than or equal to the specified one.
    pastStarttime?: StringDateTime | null | undefined; // DateTime<Utc>,
    orderColumn?: 'starttime' | 'title'; // default 'starttime';
    orderDirection?: 'asc' | 'desc'; // default 'asc';
    page?: number; // default 1;
    limit?: number; // default 10; Min(1) Max(100)
}

// ** getStreamsEvent()  **

export interface SearchStreamEventDto {
    userId?: number;
    starttime: StringDate;
    page?: number; // default 1;
    limit?: number; // default 10; Min(1) Max(100)
}

// ** getStreamsPeriod()  **

export interface SearchStreamsPeriodDto {
    userId?: number;
    start: StringDateTime;
    finish: StringDateTime;
}

export interface StreamsPeriodDto {
    date: StringDate;
    count: number;
}

// ** **