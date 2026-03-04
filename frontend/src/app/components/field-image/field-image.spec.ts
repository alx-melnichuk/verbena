import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldImage } from './field-image';

describe('FieldImage', () => {
  let component: FieldImage;
  let fixture: ComponentFixture<FieldImage>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldImage]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldImage);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
