import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, Output, ViewEncapsulation
} from "@angular/core";
import { TranslatePipe } from "@ngx-translate/core";
import { DateTimeFormatPipe } from "../../common/date-time-format-pipe";
import { Image } from "../../components/image/image";
import { StreamDto } from "../../lib-stream/stream-dto";

@Component({
    selector: 'app-panel-stream-event',
    exportAs: "appPanelStreamEvent",
    standalone: true,
    imports: [CommonModule, TranslatePipe, DateTimeFormatPipe, Image],
    templateUrl: './panel-stream-event.html',
    styleUrl: './panel-stream-event.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamEvent {
    @Input()
    public canEdit = false;
    @Input()
    public locale: string | null = null;
    @Input()
    public streamDto: StreamDto | null | undefined;

    @Output()
    readonly requestNextPage: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly viewStream: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly editStream: EventEmitter<number> = new EventEmitter();

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: "short", timeStyle: "short" };

    @HostBinding("class.app-pn-bg")
    public get isPnBg(): boolean { return true; }

    // ** Public API **

    public isFuture(start: Date | null): boolean | null {
        const currDate = new Date();
        return currDate < (start || currDate);
    }

    public doEditStream(streamId: number): void {
        if (this.canEdit && !!streamId) {
            this.editStream.emit(streamId);
        }
    }

    public doViewStream(streamId: number): void {
        if (!!streamId) {
            this.viewStream.emit(streamId);
        }
    }
}
