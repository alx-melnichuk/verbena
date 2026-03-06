import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, HostBinding, inject, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { ReactiveFormsModule, FormControl, FormGroup } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatChipsModule } from "@angular/material/chips";
import { DateAdapter, MatDateFormats, MAT_DATE_FORMATS } from "@angular/material/core";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { MatSlideToggleModule } from "@angular/material/slide-toggle";
import { MatTooltipModule } from "@angular/material/tooltip";
import { TranslatePipe } from "@ngx-translate/core";
import { CN_TITLE, CN_DESCRIPT, CN_TAG } from "../../common/fields-consts";
import { IMAGE_VALID_FILE_TYPES, MAX_FILE_SIZE } from "../../common/file-upload-consts";
import { FieldChipsInput } from "../../components/field-chips-input/field-chips-input";
import { FieldDatepicker } from "../../components/field-datepicker/field-datepicker";
import { FieldImage } from "../../components/field-image/field-image";
import { FieldInput } from "../../components/field-input/field-input";
import { FieldTextarea } from "../../components/field-textarea/field-textarea";
import { FieldTimepicker } from "../../components/field-timepicker/field-timepicker";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { ClipboardUtil } from "../../utils/clipboard.util";
import { DateUtil } from "../../utils/date.utils";
import { FileSizeUtil } from "../../utils/file_size.util";
import { ErrMsgObj } from "../../utils/http-error.util";
import { ValidFileTypesUtil } from "../../utils/valid_file_types.util";
import { StreamConfigDto } from "../stream-config-dto";
import { StreamDto, UpdateStreamFileDto, StreamDtoUtil } from "../../lib-stream/stream-dto";
import { StreamSrv } from "../../lib-stream/stream-srv";

export const PSE_DELTA_BEFORE_START = 5; // minutes
export const PSE_TIMEPICKER_INTERVAL = "1m";

interface StreamData {
    title: string;
    descript: string;
    logo: string | null;
    starttime: string | null;
    tags: string;
};

@Component({
    selector: "app-panel-stream-editor",
    exportAs: "appPanelStreamEditor",
    standalone: true,
    imports: [
        CommonModule, ReactiveFormsModule, MatButtonModule, MatChipsModule, MatFormFieldModule, MatInputModule, MatSlideToggleModule,
        MatTooltipModule, TranslatePipe,
        FieldChipsInput, FieldDatepicker, FieldInput, FieldImage, FieldTextarea, FieldTimepicker,],
    templateUrl: "./panel-stream-editor.html",
    styleUrl: "./panel-stream-editor.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamEditor implements OnChanges {
    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabledSubmit = false;
    @Input()
    public locale: string | null = null;
    @Input()
    public streamDto: StreamDto | null = null;
    @Input()
    public streamConfigDto: StreamConfigDto | null = null;

    @Output()
    readonly changeData: EventEmitter<boolean> = new EventEmitter();
    @Output()
    readonly updateStream: EventEmitter<UpdateStreamFileDto> = new EventEmitter();

    private alertSrv: AlertSrv = inject(AlertSrv);
    private streamSrv: StreamSrv = inject(StreamSrv);
    private dateAdapter: DateAdapter<Date> = inject<DateAdapter<Date>>(DateAdapter, { optional: true })!;
    private dateFormats: MatDateFormats = inject<MatDateFormats>(MAT_DATE_FORMATS, { optional: true })!;

    @HostBinding("class.global-scroll")
    public get isGlobalScroll(): boolean { return true; }

    // field-input `title`
    readonly cn_title = CN_TITLE;
    // field-textarea `descript`
    readonly cn_descript = CN_DESCRIPT;
    // field-image `logo`
    //   FieldImage parameters
    public accepts = IMAGE_VALID_FILE_TYPES;
    public maxSize = MAX_FILE_SIZE;
    public availableFileTypes: string = "";
    public availableMaxFileSize: string = "";
    //   FieldImage FormControl
    public logoFile: File | null | undefined;
    public initIsLogo: boolean = false; // original has an logo.
    // field-chips-input `tags`
    readonly cn_tag = CN_TAG;
    // field-datepicker `startDate`
    readonly minDate: Date = new Date(Date.now());
    readonly maxDate: Date = new Date(this.minDate.getFullYear(), this.minDate.getMonth() + 7, this.minDate.getDate());
    // field-timepicker `startDate`
    readonly interval = PSE_TIMEPICKER_INTERVAL;

    public controls = {
        title: new FormControl(null, []),
        descript: new FormControl(null, []),
        logo: new FormControl("", []),
        tags: new FormControl([], []),
        isStartTime: new FormControl(false, []),
        startDate: new FormControl<Date>({ value: new Date(Date.now()), disabled: true }, []),
        startTime: new FormControl<Date>({ value: new Date(Date.now()), disabled: true }, []),
        link: new FormControl("", []),
    };
    public formGroup: FormGroup = new FormGroup(this.controls);
    public isCreate = true;
    public linkForVisitors = "";
    public minTimeNow: string | null = null;

    private isChange: boolean = false;
    private origStreamDto: StreamDto = StreamDtoUtil.create();

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["streamDto"]) {
            this.prepareFormGroupByStreamDto(this.streamDto);
        }
        if (!!changes["streamConfigDto"]) {
            this.prepareFormGroupByStreamConfigDto(this.streamConfigDto);
        }
    }

    // ** Public API **

    // ** Logo file **
    public addLogoFile(file: File): void {
        this.logoFile = file;
    }
    public deleteLogoFile(): void {
        this.logoFile = (!!this.initIsLogo ? null : undefined);
        if (!!this.initIsLogo) {
            this.controls.logo.markAsDirty();
        } else {
            this.controls.logo.markAsPristine();
        }
    }
    // ** IsStartTime **
    public changeIsStartTime(isStartTime: boolean): void {
        if (!isStartTime) {
            this.controls.startDate.disable({ emitEvent: false });
            this.controls.startTime.disable();
        } else {
            this.controls.startDate.enable({ emitEvent: false });
            this.controls.startTime.enable();
        }
    }
    // ** field-datepicker **
    public updateMinTimeNow(startDate: Date | null): void {
        if (!!startDate) {
            const dateNow: Date = new Date(Date.now());
            this.minTimeNow = null;
            if (DateUtil.compare(startDate, dateNow) == 0) {
                this.minTimeNow = this.dateAdapter.format(this.getMinDateTimeNow(), this.dateFormats.display.timeInput);
            }
        }
    }

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

    public doChangeData(): void {
        const dataByDtoStr = JSON.stringify(this.getDataByDto(this.origStreamDto));
        const dataByFormStr = JSON.stringify(this.getDataByForm(this.formGroup));
        if (!this.isChange && dataByDtoStr != dataByFormStr) {
            this.isChange = true;
            this.changeData.emit(this.isChange);
        } else if (this.isChange && dataByDtoStr == dataByFormStr) {
            this.isChange = false;
            this.changeData.emit(this.isChange);
        }
    }

    public saveStream(formGroup: FormGroup): void {
        const cntlTitle = formGroup.get("title");
        const cntlDescript = formGroup.get("descript");
        const cntlTags = formGroup.get("tags");
        const cntlIsStartTime = formGroup.get("isStartTime");
        const cntlStartDate = formGroup.get("startDate");
        const cntlStartTime = formGroup.get("startTime");
        if (formGroup.pristine || formGroup.invalid || !cntlTitle || !cntlDescript || !cntlTags || !cntlIsStartTime
            || !cntlStartDate || !cntlStartTime) {
            return;
        }

        const title: string = cntlTitle.value || "";
        const descript: string = cntlDescript.value || "";
        const starttime = this.getStarttime(cntlStartDate?.value, cntlStartTime?.value) || undefined;
        const tags: string[] = cntlTags.value || [];

        const updateStreamFileDto: UpdateStreamFileDto = {
            id: (this.isCreate ? undefined : this.streamDto?.id),
            title: (this.isCreate ? title : (this.origStreamDto.title != title ? title : undefined)),
            descript: (this.isCreate ? descript : (this.origStreamDto.descript != descript ? descript : undefined)),
            starttime: (this.isCreate ? starttime : (this.origStreamDto.starttime != starttime ? starttime : undefined))?.toISOString(),
            tags: (this.isCreate ? tags : (this.origStreamDto.tags.join(",") != tags.join(",") ? tags : undefined)),
            logoFile: this.logoFile,
        };
        this.updateStream.emit(updateStreamFileDto);
    }

    public doCopyToClipboard(value: string): void {
        if (!!value) {
            const message = "panel-stream-editor.stream_link_copied_to_clipboard";
            // if (window.navigator.clipboard) {
            //     ClipboardUtil.setClipboardValue(value)
            //         .then(() => { this.alertService.showInfo(message); })
            //         .catch((err) => { ClipboardUtil.copyMessage(value); this.alertService.showInfo(message); });
            // } else {
            ClipboardUtil.copyMessage(value);
            this.alertSrv.showInfo(message);
            // }
        }
    }

    public getErrObjDate(errorObj: unknown): Record<string, unknown> {
        const result: Record<string, unknown> = {};
        const errObj = errorObj as Record<string, unknown>;
        const props = ["minValidDateTime", "maxValidDateTime"];
        for (let propName in errObj) {
            let propValue = errObj[propName];
            if (props.indexOf(propName) > -1 && !!propValue) {
                const res = this.mapDateTimes(propValue as Record<string, unknown>);
                propValue = !!res ? res : propValue;
            }
            result[propName] = propValue;
        }
        return Object.keys(result).length > 0 ? result : errObj;
    }

    // ** Private API **

    private prepareFormGroupByStreamDto(streamDto: StreamDto | null): void {
        const isDuplicate = streamDto?.id == -1;
        if (!streamDto) {
            streamDto = StreamDtoUtil.create();
        }
        this.origStreamDto = { ...streamDto };
        Object.freeze(this.origStreamDto);

        this.isCreate = (streamDto.id < 0);
        const isStartTime = (streamDto.id > 0 && !!streamDto.starttime);
        const now = new Date(Date.now());
        const year = now.getFullYear();
        const delta = PSE_DELTA_BEFORE_START;
        const currentTime = new Date(year, now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + delta, 0);
        const startDate = (!!streamDto.starttime && !isDuplicate ? new Date(streamDto.starttime.getTime()) : currentTime);
        // const startHours = ("00" + startDate.getHours()).slice(-2);
        // const startMinutes = ("00" + startDate.getMinutes()).slice(-2);
        // const startTimeStr = startHours + ":" + startMinutes;
        const link = !this.isCreate ? this.streamSrv.getLinkForVisitors(streamDto.id, true) : "";
        // Marks all descendants of `FormGroup` as `pristine` and `untouched` and sets the initial values.
        this.formGroup.reset({
            title: streamDto.title,
            descript: streamDto.descript,
            logo: streamDto.logo,
            tags: (streamDto.tags || []),
            starttime: streamDto.starttime?.toISOString(),
            isStartTime,
            startDate: startDate,
            startTime: startDate,
            link: link,
        });
        this.linkForVisitors = link;
        this.changeIsStartTime(isStartTime);
        this.updateMinTimeNow(startDate);
        this.logoFile = undefined;
        this.initIsLogo = !!streamDto.logo;
        this.isChange = false;
        if (isDuplicate) {
            this.formGroup.markAsDirty();
            this.doChangeData();
        }
    }
    private getDataByDto(streamDto: StreamDto): StreamData {
        return {
            title: streamDto.title,
            descript: streamDto.descript,
            logo: streamDto.logo,
            starttime: streamDto.starttime?.toISOString() || null,
            tags: (streamDto.tags || []).join(","),
        };
    }
    private getStarttime(startDate: Date | null | undefined, startTime: Date | null | undefined): Date | null {
        let starttime: Date | null = null;
        if (!!startDate && !!startTime) {
            starttime = new Date(startDate.getFullYear(), startDate.getMonth(), startDate.getDate()
                , startTime.getHours(), startTime.getMinutes(), startTime.getSeconds(), 0);
        }
        return starttime;
    }
    private getDataByForm(formGroup: FormGroup): StreamData {
        const starttime = this.getStarttime(formGroup.get("startDate")?.value, formGroup.get("startTime")?.value);
        return {
            title: formGroup.get("title")?.value || "",
            descript: formGroup.get("descript")?.value || "",
            logo: formGroup.get("logo")?.value || null,
            starttime: starttime?.toISOString() || null,
            tags: (formGroup.get("tags")?.value || []).join(","),
        };
    }

    private getMinDateTimeNow(): Date {
        const dateNow: Date = new Date(Date.now());
        return new Date(dateNow.setMinutes(dateNow.getMinutes() + PSE_DELTA_BEFORE_START));
    }
    // "10:12"
    /*private getStartDateTime(startDate: Date | null, startTime: string | null): Date | null {
        let startDateTime: Date | null = null;
        if (startDate != null) {
            startDateTime = new Date(startDate.getFullYear(), startDate.getMonth(), startDate.getDate(), 0, 0, 0, 0);
        }
        if (startDateTime != null && startTime != null && startTime.length > 4) {
            let { hours, minutes } = TimeUtil.parseTimeHHMM(startTime);
            startDateTime.setHours(hours);
            startDateTime.setMinutes(minutes);
            startDateTime.setSeconds(0);
            startDateTime.setMilliseconds(0);
        }
        return startDateTime;
    }*/
    private prepareFormGroupByStreamConfigDto(streamConfigDto: StreamConfigDto | null): void {
        // Set FieldImage parameters
        this.maxSize = streamConfigDto?.logoMaxSize || MAX_FILE_SIZE;
        this.accepts = (streamConfigDto?.logoValidTypes || []).join(",") || IMAGE_VALID_FILE_TYPES;
        this.availableFileTypes = ValidFileTypesUtil.text(this.accepts).join(", ").toUpperCase();
        this.availableMaxFileSize = FileSizeUtil.formatBytes(this.maxSize, 1);
    }
    private mapDateTimes(valueObj: Record<string, unknown> | null | undefined): Record<string, unknown> | null | undefined {
        const result: Record<string, unknown> = {};
        const options = { ...this.dateFormats.display.dateInput, ...this.dateFormats.display.timeInput };
        const props = ["actualDateTime", "minDateTime", "maxDateTime"];
        for (let propName in valueObj) {
            let propValue = valueObj[propName];
            if (props.indexOf(propName) > -1 && !!propValue) {
                const res = this.dateAdapter.getValidDateOrNull(this.dateAdapter.deserialize(propValue));
                propValue = !!res ? this.dateAdapter.format(res, options) : null;
            }
            result[propName] = propValue;
        }
        return Object.keys(result).length > 0 ? result : valueObj;
    }
}
