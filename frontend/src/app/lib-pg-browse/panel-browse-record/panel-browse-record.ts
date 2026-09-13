import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, Output, ViewEncapsulation } from "@angular/core";
import { DateTimeFormatPipe } from "../../common/date-time-format-pipe";
import { Image } from "../../components/image/image";
import { StreamDto } from "../../lib-stream/stream-dto";

@Component({
    selector: 'app-panel-browse-record',
    exportAs: "appPanelBrowseRecord",
    standalone: true,
    imports: [CommonModule, DateTimeFormatPipe, Image],
    templateUrl: './panel-browse-record.html',
    styleUrl: './panel-browse-record.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelBrowseRecord {
    @Input()
    public activeTag: string | null | undefined;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public stream: StreamDto | null | undefined;

    @Output()
    readonly actionView: EventEmitter<number> = new EventEmitter();

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: "short", timeStyle: "short" };

    @HostBinding("class.app-pn-bg")
    public get isPnBg(): boolean { return true; }

    // ** Public API **

    public doActionView(streamId: number): void {
        if (!!streamId) {
            this.actionView.emit(streamId);
        }
    }
}
