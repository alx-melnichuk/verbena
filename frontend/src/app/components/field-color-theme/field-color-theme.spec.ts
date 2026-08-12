import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldColorTheme } from './field-color-theme';

describe('FieldColorTheme', () => {
  let component: FieldColorTheme;
  let fixture: ComponentFixture<FieldColorTheme>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldColorTheme]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldColorTheme);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
