import { HttpErrorResponse } from "@angular/common/http";
import { PageInfo, PageInfoUtil } from "src/app/common/page-info";
import { SearchStreamDto, StreamDto, StreamListDto } from "src/app/lib-stream/stream-api.interface";
import { StreamService } from "src/app/lib-stream/stream.service";

export class StreamHandler {
    public streamLoading = false;
    public streamsDto: StreamDto[] = [];
    public pageInfo: PageInfo = PageInfoUtil.create({});

    readonly orderDirection: 'asc' | 'desc';
    readonly searchParam: 'futureStarttime' | 'pastStarttime';

    private searchDate: Date = new Date();

    constructor(
        private streamService: StreamService,
        readonly isFuture: boolean,
        readonly limit: number,
        readonly interval: number, // Minutes
    ) {
        this.orderDirection = this.isFuture ? 'asc' : 'desc';
        this.searchParam = this.isFuture ? 'futureStarttime' : 'pastStarttime';
        this.clearStream();
    }

    /** Clear array of "Future Stream". */
    public clearStream(): void {
        this.clearStreamInfo(this.limit, this.orderDirection);
    }
    /** Checking if the data needs to be updated. */
    public isNeedRefreshData(): boolean {
        const currentDate = this.datetimeByIntervals(new Date());
        return currentDate.getTime() > this.searchDate.getTime();
    }
    /** Execute a query to retrieve data from the next page of the "Stream". */
    public async searchNextStream(userId?: number | undefined): Promise<StreamListDto | HttpErrorResponse | undefined> {
        const valueMinutes = (new Date()).getMinutes() % this.interval;
        const nextPage = (valueMinutes == 0 ? 1 : this.getNextPage(this.pageInfo));
        if (!nextPage) {
            return Promise.resolve(undefined);
        }
        this.searchDate = this.datetimeByIntervals(new Date());
        const searchStream: SearchStreamDto = {
            userId,
            [this.searchParam]: this.searchDate.toISOString(),
            orderDirection: this.orderDirection,
            page: nextPage,
            limit: this.limit
        };
        let result: StreamListDto | HttpErrorResponse | undefined;
        this.streamLoading = true;
        try {
            result = await this.streamService.getStreams(searchStream);
            const streams = (result as StreamListDto);
            this.pageInfo = PageInfoUtil.create(streams);
            if (this.pageInfo.page == 1) {
                this.streamsDto = [];
            }
            this.streamsDto = this.streamsDto.concat(streams.list)
        } finally {
            this.streamLoading = false;
        }
        return result;
    }
    /** Delete a stream by its ID. */
    public async deleteDataStream(streamId: number | null): Promise<void> {
        if (!streamId) {
            return Promise.reject();
        }
        this.streamLoading = true;
        try {
            await this.streamService.deleteStream(streamId);
        } finally {
            this.streamLoading = false;
        }
    }

    private clearStreamInfo(limit: number, orderDirection: 'asc' | 'desc'): void {
        this.streamsDto = [];
        this.pageInfo = PageInfoUtil.create({ page: 0, limit, orderDirection });
    }

    private getNextPage(pageInfo: PageInfo): number | undefined {
        const nextPage = pageInfo.page + 1;
        const isNextPage = ((pageInfo.pages === -1) || (pageInfo.page !== nextPage && nextPage <= pageInfo.pages));
        return isNextPage ? nextPage : undefined;
    }

    private datetimeByIntervals(value: Date): Date {
        const starttime = new Date(value);
        const min = Math.trunc(starttime.getMinutes() / this.interval) * this.interval;
        starttime.setHours(starttime.getHours(), min, 0, 0);
        return starttime;
    }
}