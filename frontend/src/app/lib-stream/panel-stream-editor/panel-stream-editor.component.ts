import { ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnInit, Output, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ENTER, COMMA } from '@angular/cdk/keycodes';
import { ReactiveFormsModule, FormControl, Validators, FormGroup, ValidationErrors } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatChipsModule, MatChipInputEvent } from '@angular/material/chips';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslateModule } from '@ngx-translate/core';
import { MAX_FILE_SIZE, IMAGE_VALID_FILE_TYPES } from 'src/app/common/constants';
import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { CopyToClipboardUtil } from 'src/app/utils/copy-to-clipboard.util';
import { TimeUtil } from 'src/app/utils/time.util';
import { StreamDto, StreamDtoUtil, UpdateStreamFileDto } from '../stream-api.interface';
import { StreamService } from '../stream.service';

export const TAG_VALUES_MAX = 4;

@Component({
  selector: 'app-panel-stream-editor',
  standalone: true,
  imports: [
    CommonModule, MatButtonModule, MatCardModule, MatChipsModule, MatFormFieldModule, MatInputModule,  MatSlideToggleModule,
    MatDatepickerModule, MatTooltipModule, TranslateModule, ReactiveFormsModule, FieldDescriptComponent, FieldFileUploadComponent
  ],
  templateUrl: './panel-stream-editor.component.html',
  styleUrls: ['./panel-stream-editor.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamEditorComponent implements OnInit {
  @Input()
  public isDisabledSubmit = false;
  @Input()
  public streamDto: StreamDto = StreamDtoUtil.create();
  
  @Output()
  readonly updateStream: EventEmitter<UpdateStreamFileDto> = new EventEmitter();
  
  // @ViewChild(NgxMatTimepickerComponent, { static: false })
  // public timepicker: NgxMatTimepickerComponent<any> | null = null;

  public minLenTitle = 3;
  public maxLenTitle = 100;
  public minLenDescription = 3;
  public maxLenDescription = 1000;
  public countRowsDescription = 4;

  // public minDate: moment.Moment = moment().clone();
  public minDate: Date = new Date(Date.now());
  // public maxDate: moment.Moment = moment().clone().add(+6, 'month').endOf('month');
  public maxDate: Date = new Date(this.minDate.getFullYear(), this.minDate.getMonth() + 7, 0);

  public logoOrig: string | null = '';
  public logoView: string | null = '';
  public maxFileSize = MAX_FILE_SIZE;
  public validFileTypes = IMAGE_VALID_FILE_TYPES;
  public logoFile: File | undefined;

  readonly separatorKeysCodes: number[] = [ENTER, COMMA];
  readonly tagValuesMax = TAG_VALUES_MAX;
  readonly tagValues: string[] = [];
  public tagValueRemovable = true;
  public tagValueCtrl = new FormControl();

  public isCreate = true;
  
  public controls = {
    title: new FormControl(null,
      [Validators.required, Validators.minLength(this.minLenTitle), Validators.maxLength(this.maxLenTitle)]),
    descript: new FormControl(null, []),
    isStartTime: new FormControl(false, []),
    startDate: new FormControl({ value: new Date(Date.now()), disabled: true }, []),
    startTime: new FormControl('', []),
  };

  public linkForVisitors = '';

  public formGroup: FormGroup = new FormGroup(this.controls);

  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }
  

  private origStreamDto: StreamDto = StreamDtoUtil.create();
  
  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private alertService: AlertService,
    private streamService: StreamService,
  ) {
    console.log(`PanelStreamEditorComponent()`); // #-
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['streamDto'] && !!this.streamDto) {
      this.prepareData(this.streamDto);
    }
  }
  
  ngOnInit(): void {
    console.log(`PanelStreamEditorComponent().OnInit()`); // #-
  }

  // ** Public API **

  public getErrorMsg(errors: ValidationErrors | null): string {
    let result: string = '';
    const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
    for (let index = 0; index < errorsProps.length && !result; index++) {
      const error: string = errorsProps[index];
      result = !result && 'required' === error ? 'Validation.title:required' : result;
      result = !result && 'minlength' === error ? 'Validation.title:min_length' : result;
      result = !result && 'maxlength' === error ? 'Validation.title:max_length' : result;
    }
    return result;
  }

  public addFile(file: File): void {
    this.logoFile = file;
  }

  public readFile(buffFile: string[]): void {
    if (buffFile.length > 0) {
      this.logoView = buffFile[1];
      this.changeDetectorRef.markForCheck();
    }
  }

  public deleteFileLogo(): void {
    this.logoView = null;
    this.logoOrig = null;
  }

  public changeIsStartTime(): void {
    const isStartTime: boolean = !!this.controls.isStartTime.value;
    if (!isStartTime) {
      this.controls.startDate.disable({ emitEvent: false });
      this.controls.startTime.disable();
    //   if (this.timepicker !== null) { this.timepicker.disabled = true; }
    } else {
      this.controls.startDate.enable({ emitEvent: false });
      this.controls.startTime.enable();
    //   if (this.timepicker !== null) { this.timepicker.disabled = false; }
    }
  }

  public tagValueAdd(event: MatChipInputEvent): void {
    if (this.tagValues.length === 3) { return; }
    const input = event.input; // ?!
    const value = event.value;
    if ((value || '').trim()) {
      this.tagValues.push(value.trim());
    }
    if (input) {
      input.value = '';
    }
    this.tagValueCtrl.setValue(null);
  }

  public tagValueRemove(tagValueRemove: string): void {
    const index = this.tagValues.indexOf(tagValueRemove);
    if (index >= 0) {
      this.tagValues.splice(index, 1);
    }
  }

  public saveStreamCard(): void {
    let startDateTime: Date | null = null;
    const isStartTime: boolean = !!this.controls.isStartTime.value;
    if (isStartTime) {
      // d1.toISOString() // '2024-01-25T14:14:37.470Z'
      // d1.toJSON()      // '2024-01-25T14:14:37.470Z'
      startDateTime = this.getStartDateTime(this.controls.startDate.value, this.controls.startTime.value);
      //   const timeVal: moment.Moment = moment(this.controls.startTime.value);
      //   beginDate.set({ hour: timeVal.get('hour'), minute: timeVal.get('minute'), second: timeVal.get('second') });
      //   startTimeStr = beginDate.format(MOMENT_ISO8601);
    }
    const title: string | undefined = this.controls.title.value || undefined;
    // const descript: string | undefined = this.getValue(this.controls.descript.value, this.origStreamDto.descript);
    const descript: string | undefined = this.controls.descript.value || undefined;
    const len = this.tagValues.length;
    const tags = this.tagValues.slice(0, (len > 3 ? 3 : len));

    const updateStreamFileDto: UpdateStreamFileDto = {};
    
    if (this.streamDto.id < 0) { // Mode: "create"
      updateStreamFileDto.createStreamDto = {
        title: (title || ''),
        descript,
        starttime: (startDateTime != null ? startDateTime.toJSON() : undefined),
        tags,
      };
    } else { // Mode: "update"
        updateStreamFileDto.id = this.streamDto.id;
        updateStreamFileDto.modifyStreamDto = {
          title: (this.origStreamDto.title != title ? title : undefined),
          descript: (this.origStreamDto.descript != descript ? descript : undefined),
          starttime: (startDateTime != null ? startDateTime.toJSON() : undefined),
          tags,
        }
    }
    updateStreamFileDto.logoFile = this.logoFile;
    this.updateStream.emit(updateStreamFileDto);
  }

  public doCopyToClipboard(value: string): void {
    if (!!value) {
      CopyToClipboardUtil.copyMessage(value);
      this.alertService.showInfo('stream_edit.stream_link_copied_to_clipboard');
    }
  }

  // ** Private API **

  private prepareData(streamDto: StreamDto): void {
    if (!streamDto) {
      return;
    }
    this.origStreamDto = { ...streamDto };
    Object.freeze(this.origStreamDto);
    // moment().add(+5, 'minute');
    const now = new Date(Date.now())
    const currentTime = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + 5, now.getSeconds());
    // ?? const starttime = (!!streamDto.starttime ? moment(streamDto.starttime, MOMENT_ISO8601) : currentTime);
    // Date.parse("2019-01-01T00:00:00.000Z");
    const startDate = (!!streamDto.starttime ? new Date(Date.parse(streamDto.starttime)) : currentTime);
    const startHours = ('00' + startDate.getHours()).slice(-2);
    const startMinutes = ('00' + startDate.getMinutes()).slice(-2);
    const startSeconds = ('00' + startDate.getSeconds()).slice(-2);
    const startTimeStr = startHours + ':' + startMinutes + ':' + startSeconds;
    this.formGroup.patchValue({
      title: streamDto.title,
      descript: streamDto.descript,
      starttime: streamDto.starttime,
      isStartTime: (streamDto.id > 0 && !!streamDto.starttime),
      startDate: startDate,
      startTime: startTimeStr,
    });
    this.linkForVisitors = this.streamService.getLinkForVisitors(streamDto.id, true);
    this.changeIsStartTime();
    this.tagValues.length = 0;
    this.tagValues.push(...streamDto.tags);
    this.logoView = streamDto.logo;
    this.logoOrig = streamDto.logo;
    this.isCreate = (streamDto.id < 0);
    this.controls.startTime.markAsPristine(); // ?
    this.formGroup.markAsPristine(); // ?
  }
  private getValue(value: string | null | undefined, origValue: string | null | undefined): string | undefined {
    return !!value && origValue != value ? value : undefined;
  }
  // '10:12'
  private getStartDateTime(startDate: Date | null, startTime: string | null): Date | null {
    let startDateTime: Date | null = null;
    if (startDate != null) {
      startDateTime = new Date(startDate.getFullYear(), startDate.getMonth(), startDate.getDate(), 0, 0, 0, 0);
    }
    if (startDateTime != null && startTime != null && startTime.length > 4) {
        let { hours, minutes } = TimeUtil.parseTimeHHMM(startTime);
        // const hoursStr = startTime.slice(0,2);
        // const hours = parseInt(hoursStr, 10);
        startDateTime.setHours(hours);
        // const minutesStr = startTime.slice(3,6);
        // const minutes = parseInt(minutesStr, 10);
        startDateTime.setMinutes(minutes);
        startDateTime.setSeconds(0);
        startDateTime.setMilliseconds(0);
      }
    return startDateTime;
  }
}
