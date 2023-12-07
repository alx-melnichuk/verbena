import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgViewComponent } from './pg-view.component';

describe('PgViewComponent', () => {
  let component: PgViewComponent;
  let fixture: ComponentFixture<PgViewComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PgViewComponent],
    });
    fixture = TestBed.createComponent(PgViewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
