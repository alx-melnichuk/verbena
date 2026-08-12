import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslatePipe, TranslateService } from "@ngx-translate/core";
import { DateTimeFormatPipe } from "../../common/date-time-format-pipe";
import { Spinner } from "../../components/spinner/spinner";
import { ViewItemList } from "../../components/view-item-list/view-item-list";
import { ItemViewPage, ViewItemListByPage } from "../../components/view-item-list/view-item-list-by-page";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { StreamsPeriodDto } from "../../lib-stream/stream-dto";
import { DateUtil } from "../../utils/date.utils";
import { PanelStreamCalendar } from "../panel-stream-calendar/panel-stream-calendar";
import { PanelStreamEvent } from "../panel-stream-event/panel-stream-event";
import { PanelStreamInfo } from "../panel-stream-info/panel-stream-info";

const CN_DEFAULT_LIMIT = 10;

@Component({
    selector: "app-panel-stream-list",
    exportAs: "appPanelStreamList",
    standalone: true,
    imports: [CommonModule, TranslatePipe, MatButtonModule, MatTooltipModule, Spinner, DateTimeFormatPipe,
        PanelStreamCalendar, PanelStreamEvent, PanelStreamInfo, ViewItemList, ViewItemListByPage,
    ],
    templateUrl: "./panel-stream-list.html",
    styleUrl: "./panel-stream-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamList implements OnChanges {
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    @Input()
    public clndDaySelected: Date | null | undefined;
    @Input()
    public clndIsLoading: boolean | null | undefined;
    @Input()
    public clndMarkedDates: StreamsPeriodDto[] = [];
    @Input()
    public clndMaxDate: Date | null | undefined;
    @Input()
    public clndMinDate: Date | null | undefined;

    @Input()
    public strmFtrDeleteIds: number[] = []; // A list of item IDs that have been removed from the list.
    @Input()
    public strmFtrIsLoading: boolean | null | undefined;
    @Input()
    public strmFtrIsReset: boolean | null | undefined;
    @Input()
    public strmFtrItemPage: ItemViewPage | null | undefined;
    @Input()
    public strmFtrMaxSizeRows: number = CN_DEFAULT_LIMIT * 3;
    @Input()
    public strmFtrRowsOnPage: number = CN_DEFAULT_LIMIT;

    @Input()
    public strmPstDeleteIds: number[] = []; // A list of item IDs that have been removed from the list.
    @Input()
    public strmPstIsLoading: boolean | null | undefined;
    @Input()
    public strmPstIsReset: boolean | null | undefined;
    @Input()
    public strmPstItemPage: ItemViewPage | null | undefined;
    @Input()
    public strmPstMaxSizeRows: number = CN_DEFAULT_LIMIT * 3;
    @Input()
    public strmPstRowsOnPage: number = CN_DEFAULT_LIMIT;

    @Input()
    public evntDeleteIds: number[] = []; // A list of item IDs that have been removed from the list.
    @Input()
    public evntIsLoading: boolean | null | undefined;
    @Input()
    public evntIsReset: boolean | null | undefined;
    @Input()
    public evntItemPage: ItemViewPage | null | undefined;
    @Input()
    public evntMaxSizeRows: number = CN_DEFAULT_LIMIT * 3;
    @Input()
    public evntRowsOnPage: number = CN_DEFAULT_LIMIT

    @Input()
    public locale: string | null | undefined;

    @Output()
    readonly calendarForPeriod: EventEmitter<Date> = new EventEmitter();
    @Output()
    readonly loadFuturePage: EventEmitter<{ page: number, limit: number }> = new EventEmitter();
    @Output()
    readonly loadPastPage: EventEmitter<{ page: number, limit: number }> = new EventEmitter();
    @Output()
    readonly loadEventDatePage: EventEmitter<{ date: Date | null, page: number, limit: number }> = new EventEmitter();

    @Output()
    readonly resetInfo: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly actionDuplicate: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionEdit: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionDelete: EventEmitter<{ isFuture: boolean, id: number }> = new EventEmitter();

    public calendarMonth: Date = new Date();

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: "long" };

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["clndDaySelected"] && !!this.clndDaySelected) {
            let date = new Date(this.clndDaySelected.getTime());
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

    public isShowEvents(clndDaySelected: Date | null | undefined, calendarMonth: Date): boolean {
        return !!clndDaySelected ? DateUtil.compareYearMonth(clndDaySelected, calendarMonth) == 0 : false;
    }

    public doLoadEventDatePage(selectedDate: Date | null, page: number, limit: number): void {
        this.loadEventDatePage.emit({ date: selectedDate, page, limit });
    }

    // ** "Future Stream" and "Past Stream" panel-stream-info **

    public doLoadFuturePage(page: number, limit: number): void {
        this.loadFuturePage.emit({ page, limit });
    }

    public doLoadPastPage(page: number, limit: number): void {
        this.loadPastPage.emit({ page, limit });
    }

    // ** **

    public doResetInfo(): void {
        this.resetInfo.emit();
    }

    public doActionDuplicate(streamId: number): void {
        this.actionDuplicate.emit(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.actionEdit.emit(streamId);
    }

    public doActionDelete(isFuture: boolean, info: { id: number, title: string }): void {
        if (!info || !info.id) {
            return;
        }
        const message = this.translateSrv.instant("panel-stream-list.sure_you_want_delete_stream", { title: info.title });
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.actionDelete.emit({ isFuture, id: info.id });
                }
            });
    }

    // ** Private API **
}
