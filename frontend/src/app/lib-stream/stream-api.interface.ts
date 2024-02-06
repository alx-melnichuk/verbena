export interface StreamApi {
}
export type StringDate = string;
export type StringDateTime = string;   

export enum StreamState {
  Waiting = 'Waiting',
  Preparing = 'Preparing',
  Started = 'Started',
  Stopped = 'Stopped',
  Paused = 'Paused'
}
export class StreamStateUtil {
  public static create(value: string): StreamState | null {
    let result: StreamState | null = null;
    switch (value) {
      case StreamState.Waiting: result = StreamState.Waiting; break;
      case StreamState.Preparing: result = StreamState.Preparing; break;
      case StreamState.Started: result = StreamState.Started; break;
      case StreamState.Stopped: result = StreamState.Stopped; break;
      case StreamState.Paused: result = StreamState.Paused; break;
    }
    return result;
  }
  public static isActive(streamState: StreamState): boolean {
    return [StreamState.Preparing, StreamState.Started, StreamState.Paused].includes(streamState);
  }
} 

export type StreamSateType = 'Waiting' | 'Preparing' | 'Started' | 'Stopped' | 'Paused';

export interface StreamDto {
  id: number;
  userId: number;
  title: string;
  descript: string; // description
  logo: string | null;
  starttime: StringDateTime | null;
  live: boolean;
  state: StreamState; // ['Waiting', 'Preparing', 'Started', 'Stopped', 'Paused']
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
      state: (streamDto?.state || StreamState.Waiting),
      tags: (streamDto?.tags || []),
      source: (streamDto?.source || 'obs'),
      isMyStream: (streamDto?.isMyStream),
      // credentials: (streamDto?.credentials || null),
      // publicTarget: (streamDto?.publicTarget || null)
      createdAt: (streamDto?.createdAt || ''),
      updatedAt: (streamDto?.updatedAt || ''),
    };
  }
  // public static isFuture(startTime: StringDateTime | null): boolean | null {
  //   return (!!startTime ? moment().isBefore(moment(startTime, MOMENT_ISO8601), 'day') : null);
  // }
}

export interface ModifyStreamDto {
  title?: string | undefined;
  descript?: string | undefined;
  starttime?: StringDateTime | null | undefined;
  source?: string | undefined;
  tags?: string[] | undefined;
} 

export interface CreateStreamDto {
  title: string;
  descript?: string | undefined;
  starttime?: StringDateTime | undefined;
  source?: string | undefined;
  tags: string[];
} 

export interface UpdateStreamFileDto {
  id?: number | undefined;
  modifyStreamDto?: ModifyStreamDto | undefined;
  createStreamDto?: CreateStreamDto | undefined;
  logoFile?: File | undefined;
}