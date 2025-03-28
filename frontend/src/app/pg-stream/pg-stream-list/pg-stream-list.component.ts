import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';

import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';
import { StreamCalendarService } from 'src/app/lib-stream/stream-calendar.service';
import { StreamListService } from 'src/app/lib-stream/stream-list.service';
import { CN_DELETE_STREAM, StreamListComponent } from 'src/app/lib-stream/stream-list/stream-list.component';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

@Component({
    selector: 'app-pg-stream-list',
    standalone: true,
    imports: [CommonModule, StreamListComponent],
    templateUrl: './pg-stream-list.component.html',
    styleUrl: './pg-stream-list.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamListComponent implements OnInit {
    public errObj: KeyValue<string, string> | null = null;
    public isRefreshStreamEvent: boolean | null | undefined;
    public profileDto: ProfileDto | null;

    constructor(
        private changeDetector: ChangeDetectorRef,
        private route: ActivatedRoute,
        private streamService: StreamService,
        public streamListService: StreamListService,
        public streamCalendarService: StreamCalendarService,
    ) {
        this.profileDto = this.route.snapshot.data['profileDto'];
    }

    async ngOnInit(): Promise<void> {
        await this.loadFutureAndPastStreamsAndSchedule();
    }
    // ** Public API **

    // ** "Streams Calendar" panel-stream-calendar **

    public async doCalendarForPeriod(calendarStart: Date): Promise<void> {
        const buffPromise: Promise<unknown>[] = [];
        buffPromise.push( // Get a list of events (streams) for a specified date.
            this.streamCalendarService.getCalendarInfoForPeriod(calendarStart, false)
        );
        if (this.streamCalendarService.isShowEvents(calendarStart)) {
            buffPromise.push( // Get the next page of the list of short streams for the selected date.
                this.streamCalendarService.getListEventsForDate(this.streamCalendarService.eventsOfDaySelected, 1)
            );
        } else {
            this.streamCalendarService.clearStreamsEvent();
        }
        try {
            await Promise.all(buffPromise)
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** "Streams Event" panel-stream-event **

    public async doStreamEventsForDate(info: { selectedDate: Date | null, pageNum: number }): Promise<void> {
        try {
            // Get the next page of the list of short streams for the selected date.
            await this.streamCalendarService.getListEventsForDate(info.selectedDate, info.pageNum);
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** "Future Stream" and "Past Stream" panel-stream-info **

    public async doSearchNextFutureStream(): Promise<void> {
        try {
            // Get the next page of the "Future Stream".
            await this.streamListService.searchNextFutureStream();
            // Refresh view for "panel-stream-event".
            this.isRefreshStreamEvent = !this.isRefreshStreamEvent ? true : undefined;
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    public async doSearchNextPastStream(): Promise<void> {
        try {
            // Get the next page of "Past Stream".
            await this.streamListService.searchNextPastStream();
            // Refresh view for "panel-stream-event".
            this.isRefreshStreamEvent = !this.isRefreshStreamEvent ? true : undefined;
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** **

    public doActionDuplicate(streamId: number): void {
        this.streamService.redirectToStreamCreationPage(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.streamService.redirectToStreamEditingPage(streamId);
    }

    public doActionView(streamId: number): void {
        this.streamService.redirectToStreamViewPage(streamId);
    }

    public async doActionDelete(streamId: number): Promise<void> {
        await this.deleteDataStream(streamId);
    }

    // ** Private API **

    private async loadFutureAndPastStreamsAndSchedule(): Promise<void> {
        // Clear array of "Future Stream"
        this.streamListService.clearFutureStream();
        // Clear array of "Past Stream"
        this.streamListService.clearPastStream();

        const buffPromise: Promise<unknown>[] = [];
        const now = new Date();
        now.setHours(0, 0, 0, 0);

        // Get the next page of the "Future Stream".
        buffPromise.push(this.streamListService.searchNextFutureStream());
        // Get the next page of "Past Stream".
        buffPromise.push(this.streamListService.searchNextPastStream());
        const selectedDate = this.streamCalendarService.eventsOfDaySelected || now;
        // Get a list of short streams for the selected date.
        buffPromise.push(this.streamCalendarService.getListEventsForDate(selectedDate, 1));
        const selectedMonth = this.streamCalendarService.calendarMonth || now;
        // Get a list of events (streams) for a specified date.
        buffPromise.push(this.streamCalendarService.getCalendarInfoForPeriod(selectedMonth, true));

        this.changeDetector.markForCheck();
        try {
            await Promise.all(buffPromise);
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    private async deleteDataStream(streamId: number): Promise<void> {
        this.errObj = null;
        this.changeDetector.markForCheck();
        try {
            // Delete a stream by its ID.
            await this.streamListService.deleteDataStream(streamId);
            // Update the list of next and past streams.
            await this.loadFutureAndPastStreamsAndSchedule();
        } catch (error) {
            if (error instanceof HttpErrorResponse) {
                const value = HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0];
                this.errObj = { "key": CN_DELETE_STREAM, value };
            }
        } finally {
            this.changeDetector.markForCheck();
        }
    }
}
