import { CommonModule } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, Input, Output, EventEmitter, HostBinding
} from "@angular/core";
import { AlertMode } from "./alert-info";

@Component({
    selector: "app-alert",
    exportAs: "appAlert",
    standalone: true,
    imports: [CommonModule],
    templateUrl: "./alert.html",
    styleUrl: "./alert.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Alert {
    @Input()
    public isOneLine: boolean | undefined = false;
    @Input()
    public closeable: boolean | undefined = true;
    @Input()
    public mode: AlertMode | undefined = AlertMode.comment;
    @Input()
    public title: string | undefined;
    @Input()
    public message: string | undefined;
    @Input()
    public messageHtml: string | undefined;

    @Output()
    readonly closed: EventEmitter<void> = new EventEmitter();

    @HostBinding("class.app-comment")
    public get isComment(): boolean {
        return this.mode === AlertMode.comment;
    }

    @HostBinding("class.app-info")
    public get isInfo(): boolean {
        return this.mode === AlertMode.info;
    }

    @HostBinding("class.app-warning")
    public get isWarning(): boolean {
        return this.mode === AlertMode.warning;
    }

    @HostBinding("class.app-error")
    public get isError(): boolean {
        return this.mode === AlertMode.error;
    }

    @HostBinding("class.app-success")
    public get isSuccess(): boolean {
        return this.mode === AlertMode.success;
    }

    // ** Public API **

    public doClose(): void {
        this.closed.emit();
    }
}
