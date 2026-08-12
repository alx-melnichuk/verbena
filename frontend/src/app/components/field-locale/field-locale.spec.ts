import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldLocale } from './field-locale';

describe('FieldLocale', () => {
  let component: FieldLocale;
  let fixture: ComponentFixture<FieldLocale>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldLocale]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldLocale);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
