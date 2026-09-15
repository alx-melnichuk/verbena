import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBrowseBanned } from './pg-browse-banned';

describe('PgBrowseBanned', () => {
  let component: PgBrowseBanned;
  let fixture: ComponentFixture<PgBrowseBanned>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBrowseBanned]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBrowseBanned);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
