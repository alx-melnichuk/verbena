import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { TranslatePipe, TranslateService } from "@ngx-translate/core";
import { DateTimeFormatPipe } from "../../common/date-time-format-pipe";
import { LocaleSrv } from "../../common/locale-srv";
import { Spinner } from "../../components/spinner/spinner";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { DateUtil } from "../../utils/date.utils";
import { PanelStreamCalendar } from "../panel-stream-calendar/panel-stream-calendar";
import { PanelStreamEvent } from "../panel-stream-event/panel-stream-event";
import { PanelStreamInfo } from "../panel-stream-info/panel-stream-info";
import { StreamsPeriodDto, StreamDto, StreamEventDto } from "../stream-dto";

@Component({
    selector: "app-panel-stream-list",
    exportAs: "appPanelStreamList",
    standalone: true,
    imports: [CommonModule, TranslatePipe, Spinner, DateTimeFormatPipe, PanelStreamCalendar, PanelStreamEvent, PanelStreamInfo],
    templateUrl: "./panel-stream-list.html",
    styleUrl: "./panel-stream-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamList implements OnChanges {
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

    private translateService: TranslateService = inject(TranslateService);
    private dialogSrv: DialogSrv = inject(DialogSrv);

    public calendarMonth: Date = new Date();
    public isLoadData: boolean = false;

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: "long" };

    public localeSrv: LocaleSrv = inject(LocaleSrv);

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["calendarDaySelected"] && !!this.calendarDaySelected) {
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
        const message = this.translateService.instant("panel-stream-list.sure_you_want_delete_stream", { title: info.title });
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.actionDelete.emit(info.id);
                }
            });
    }

    // ** Private API **
}
