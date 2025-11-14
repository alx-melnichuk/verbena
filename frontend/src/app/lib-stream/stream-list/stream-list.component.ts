import { CommonModule, KeyValue } from '@angular/common';
import {
    ChangeDetectionStrategy, Component, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { LocaleService } from 'src/app/common/locale.service';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';

import { PanelStreamCalendarComponent } from '../panel-stream-calendar/panel-stream-calendar.component';
import { PanelStreamEventComponent } from '../panel-stream-event/panel-stream-event.component';
import { PanelStreamInfoComponent } from '../panel-stream-info/panel-stream-info.component';
import { StreamDto, StreamEventDto, StreamsPeriodDto } from '../stream-api.interface';
import { DateUtil } from 'src/app/utils/date.utils';

export const CN_DELETE_STREAM = "deleteStream";

@Component({
    selector: 'app-stream-list',
    standalone: true,
    imports: [CommonModule, TranslatePipe, PanelStreamEventComponent, PanelStreamInfoComponent,
        PanelStreamCalendarComponent, DateTimeFormatPipe, SpinnerComponent],
    templateUrl: './stream-list.component.html',
    styleUrl: './stream-list.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamListComponent implements OnChanges {
    @Input()
    public errObj: KeyValue<string, string> | null = null;
    @Input()
    public calendarDaySelected: Date | null = null;
    @Input()
    public calendarMarkedDates: StreamsPeriodDto[] = [];
    @Input()
    public calendarMaxDate: Date | null = null;
    @Input()
    public calendarMinDate: Date | null = null;
    @Input()
    public futureStreams: StreamDto[] = [];
    @Input()
    public isLoadStreams = false;
    @Input()
    public isRefreshStreamEvent: boolean | null | undefined;
    @Input()
    public pastStreams: StreamDto[] = [];
    @Input()
    public profileDto: ProfileDto | null = null;
    @Input()
    public streamEventsForDay: StreamEventDto[] = [];

    @Output()
    readonly calendarForPeriod: EventEmitter<Date> = new EventEmitter();
    @Output()
    readonly futureNextPage: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly pastNextPage: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly streamEventsForDate: EventEmitter<{ selectedDate: Date | null, pageNum: number }> = new EventEmitter();

    @Output()
    readonly actionDuplicate: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionEdit: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionView: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionDelete: EventEmitter<number> = new EventEmitter();

    public calendarMonth: Date = new Date();
    public isLoadData: boolean = false;

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'long' };

    public localeService: LocaleService = inject(LocaleService);

    private translateService: TranslateService = inject(TranslateService);
    private dialogService: DialogService = inject(DialogService);
    private alertService: AlertService = inject(AlertService);

    constructor() {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['errObj'] && !!this.errObj) {
            this.alertService.showError(this.errObj.value, this.getTitleByKey(this.errObj.key));
        }
        if (!!changes['calendarDaySelected'] && !!this.calendarDaySelected) {
            let date = new Date(this.calendarDaySelected.getTime());
            date.setHours(0, 0, 0, 0);
            const startMonth = DateUtil.dateFirstDayOfMonth(date);
            this.calendarMonth = startMonth;
        }
    }

    // ** Public API **

    // ** "Streams Calendar" panel-stream-calendar **

    public doCalendarForPeriod(calendarStart: Date): void {
        this.calendarMonth = calendarStart;
        this.calendarForPeriod.emit(calendarStart);
    }

    // ** "Streams Event" panel-stream-event **

    public isShowEvents(calendarDaySelected: Date | null, calendarMonth: Date): boolean {
        return !!calendarDaySelected ? DateUtil.compareYearMonth(calendarDaySelected, calendarMonth) == 0 : false;
    }

    public doStreamEventsForDate(selectedDate: Date | null, pageNum: number): void {
        this.streamEventsForDate.emit({ selectedDate, pageNum });
    }

    // ** "Future Stream" and "Past Stream" panel-stream-info **

    public doFutureNextPage(): void {
        this.futureNextPage.emit();
    }

    public doPastNextPage(): void {
        this.pastNextPage.emit();
    }

    // ** **

    public doActionDuplicate(streamId: number): void {
        this.actionDuplicate.emit(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.actionEdit.emit(streamId);
    }

    public doActionView(streamId: number): void {
        this.actionView.emit(streamId);
    }

    public doActionDelete(info: { id: number, title: string }): void {
        if (!info || !info.id) {
            return;
        }
        const message = this.translateService.instant('stream_list.sure_you_want_delete_stream', { title: info.title });
        this.dialogService.openConfirmation(message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' })
            .then((res) => {
                if (!!res) {
                    this.actionDelete.emit(info.id);
                }
            });
    }

    // ** Private API **

    private getTitleByKey(errKey: string): string {
        let result = '';
        if (CN_DELETE_STREAM == errKey) {
            result = 'stream_list.error_delete_stream';
        }
        return result;
    }
}
