import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnChanges, Output, 
  SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { ENTER, COMMA } from '@angular/cdk/keycodes';
import { ReactiveFormsModule, FormControl, Validators, FormGroup, ValidationErrors } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
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
import { FieldTimeComponent } from 'src/app/components/field-time/field-time.component';
import { StringDateTime } from 'src/app/common/string-date-time';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { CopyToClipboardUtil } from 'src/app/utils/copy-to-clipboard.util';
import { TimeUtil } from 'src/app/utils/time.util';

import { StreamService } from '../stream.service';
import { StreamDto, StreamDtoUtil, UpdateStreamFileDto } from '../stream-api.interface';

export const TAG_VALUES_MAX = 4;

@Component({
  selector: 'app-panel-stream-editor',
  standalone: true,
  imports: [
    CommonModule, MatButtonModule, MatChipsModule, MatFormFieldModule, MatInputModule,  MatSlideToggleModule,
    MatDatepickerModule, MatTooltipModule, TranslateModule, ReactiveFormsModule, FieldDescriptComponent, FieldFileUploadComponent,
    FieldTimeComponent
  ],
  templateUrl: './panel-stream-editor.component.html',
  styleUrls: ['./panel-stream-editor.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamEditorComponent implements OnChanges {
  @Input()
  public isDisabledSubmit = false;
  @Input()
  public streamDto: StreamDto = StreamDtoUtil.create();
  
  @Output()
  readonly updateStream: EventEmitter<UpdateStreamFileDto> = new EventEmitter();
  @Output()
  readonly cancelStream: EventEmitter<void> = new EventEmitter();
  
  public minLenTitle = 3;
  public maxLenTitle = 100;
  public minLenDescription = 3;
  public maxLenDescription = 1000;
  public countRowsDescription = 4;

  public minDate: Date = new Date(Date.now());
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
    tagValue: this.tagValueCtrl,
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
    } else {
      this.controls.startDate.enable({ emitEvent: false });
      this.controls.startTime.enable();
    }
  }

  public tagValueAdd(event: MatChipInputEvent): void {
    const input = event.input; // ?!
    const value = event.value;
    if (!!value) {
      const val = (value || '').trim();
      if (!!val && this.tagValues.length < TAG_VALUES_MAX && !this.tagValues.includes(val)) {
        this.tagValues.push(val);
      }
      if (input) {
        input.value = '';
      }
    }
    this.setTagValueDirtyOrPristine(this.origStreamDto.tags, this.tagValues, this.tagValueCtrl);
  }

  public tagValueRemove(tagValueRemove: string): void {
    const index = this.tagValues.indexOf(tagValueRemove);
    if (index >= 0) {
      this.tagValues.splice(index, 1);
      this.setTagValueDirtyOrPristine(this.origStreamDto.tags, this.tagValues, this.tagValueCtrl);
    }
  }

  public cancelStreamCard(): void {
    this.cancelStream.emit();
  }

  public saveStreamCard(): void {
    let starttime: StringDateTime | undefined;
    const isStartTime: boolean = !!this.controls.isStartTime.value;
    if (isStartTime) {
      const startDateTime = this.getStartDateTime(this.controls.startDate.value, this.controls.startTime.value);
      starttime = !!startDateTime ? startDateTime.toISOString() : undefined;
    }
    const title: string | undefined = this.controls.title.value || undefined;
    const descript: string | undefined = this.controls.descript.value || undefined;
    const len = this.tagValues.length;
    const tags = this.tagValues.slice(0, (len > TAG_VALUES_MAX ? TAG_VALUES_MAX : len));

    const updateStreamFileDto: UpdateStreamFileDto = {};
    
    if (this.streamDto.id < 0) { // Mode: "create"
      updateStreamFileDto.createStreamDto = {
        title: (title || ''),
        descript,
        starttime,
        tags,
      };
    } else { // Mode: "update"
        updateStreamFileDto.id = this.streamDto.id;
        updateStreamFileDto.modifyStreamDto = {
          title: (this.origStreamDto.title != title ? title : undefined),
          descript: (this.origStreamDto.descript != descript ? descript : undefined),
          starttime: (this.origStreamDto.starttime != starttime ? starttime : undefined),
          tags: (JSON.stringify(this.origStreamDto.tags) != JSON.stringify(tags) ? tags : undefined),
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
    const now = new Date(Date.now())
    const currentTime = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + 5, now.getSeconds());
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
  }
  // '10:12'
  private getStartDateTime(startDate: Date | null, startTime: string | null): Date | null {
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
  }

  private setTagValueDirtyOrPristine(old_tags: string[], new_tags: string[], tagValueCtrl: FormControl<any>): void {
    const isDirty = JSON.stringify(old_tags) != JSON.stringify(new_tags);
    if (!tagValueCtrl) {
      return;
    }
    if (isDirty) {
      tagValueCtrl.markAsDirty();
    } else {
      tagValueCtrl.markAsPristine();
    }
  }
}
