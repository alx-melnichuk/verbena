import { StringDate, StringDateTime } from '../common/string-date-time';
import { StringDateTimeUtil } from '../utils/string-date-time.util';


export enum StreamState {
  waiting = 'waiting',
  preparing = 'preparing',
  started = 'started',
  stopped = 'stopped',
  paused = 'paused'
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

export type StreamSateType = 'waiting' | 'preparing' | 'Started' | 'Stopped' | 'Paused';

export interface StreamDto {
  id: number;
  userId: number;
  title: string;
  descript: string; // description
  logo: string | null;
  starttime: StringDateTime | null;
  live: boolean;
  state: StreamState; // ['waiting', 'Preparing', 'Started', 'Stopped', 'Paused']
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

/*pub struct StreamInfoDto {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub descript: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(with = "serial_datetime")]
    pub starttime: DateTime<Utc>,
    pub live: bool,
    pub state: StreamState,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>,
    #[rustfmt::skip]
    #[serde(default, with = "serial_datetime_option", skip_serializing_if = "Option::is_none")]
    pub stopped: Option<DateTime<Utc>>,
    pub status: bool,
    pub source: String,
    pub tags: Vec<String>,
    pub is_my_stream: bool,
    #[serde(with = "serial_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "serial_datetime")]
    pub updated_at: DateTime<Utc>,
}*/

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
/*pub struct SearchStreamInfoResponseDto {
    pub list: Vec<StreamInfoDto, Global>,
    pub limit: u32,
    pub count: u32,
    pub page: u32,
    pub pages: u32,
}*/

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
  isFuture?: boolean; // true-'future', false-'past'
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