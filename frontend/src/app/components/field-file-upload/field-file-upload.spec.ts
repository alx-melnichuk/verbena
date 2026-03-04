import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldFileUpload } from './field-file-upload';

describe('FieldFileUpload', () => {
  let component: FieldFileUpload;
  let fixture: ComponentFixture<FieldFileUpload>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldFileUpload]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldFileUpload);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
